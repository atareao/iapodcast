mod models;

use minijinja::context;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter
};
use tracing::{debug, error, info};
use std::str::FromStr;

use models::{
    publisher::{
        Telegram,
        get_telegram_client,
        Mastodon,
        get_mastodon_client,
    },
    episode::Episode,
    config::{
        Configuration,
        Post,
        Page,
    },
    ENV,
};

#[tokio::main]
async fn main() {
    let log_level = option_env!("RUST_LOG").unwrap_or("DEBUG");
    let configuration = Configuration::read_configuration().await;

    tracing_subscriber::registry()
        .with(EnvFilter::from_str(log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("Configuration: {:?}", configuration);

    update(&configuration).await;

    let posts = read_episodes().await;
    let pages = read_pages().await;
    debug!("{:?}", posts);
    if posts.is_empty() {
        debug!("=== No audios found ===");
    } else {
        debug!("=== Generation ===");
        create_public(&configuration).await;
        generate_html(&configuration, &posts, &pages).await;
        generate_index(&configuration, &posts, &pages).await;
        generate_pages(&configuration, &pages).await;
        generate_feed(&configuration, &posts).await;
        generate_stats(&configuration, &posts, &pages).await;
        let public = if configuration.get_podcast().base_url.is_empty() {
            configuration.get_public().to_owned()
        } else {
            format!(
                "{}/{}",
                configuration.get_public(),
                configuration.get_podcast().base_url
            )
        };
        //TODO: Copy directory assets a /public/{podcast}/assets
        //let output = format!("{}/style.css", public);
        let assets_dir = format!("{}/assets", public);
        create_dir(&assets_dir).await;
        copy_all_files(configuration.get_assets(), &assets_dir).await;
    }
}

async fn read_episodes() -> Vec<Post> {
    let mut posts = Vec::new();
    let mut episodes_dir = tokio::fs::read_dir("episodes").await.unwrap();
    while let Some(file) = episodes_dir.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_file() {
            let filename = file.file_name().to_str().unwrap().to_string();
            if filename.ends_with(".md") {
                debug!("Read episode: {}", filename);
                match Episode::new(&filename).await {
                    Ok(episode) => posts.push(episode.get_post()),
                    Err(err) => {
                        error!("Can not write {}. {:#}", filename, err);
                        // render causes as well
                        let mut err = &err as &dyn std::error::Error;
                        while let Some(next_err) = err.source() {
                            error!("caused by: {:#}", next_err);
                            err = next_err;
                        }
                    }
                }
            }
        }
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

async fn read_pages() -> Vec<Post> {
    let mut posts = Vec::new();
    if let Ok(mut pages_dir) = tokio::fs::read_dir("pages").await {
        while let Some(file) = pages_dir.next_entry().await.unwrap() {
            if file.metadata().await.unwrap().is_file() {
                let filename = file.file_name().to_str().unwrap().to_string();
                if filename.ends_with(".md") {
                    debug!("Read pages: {}", filename);
                    match Page::new(&filename).await {
                        Ok(episode) => posts.push(episode.get_post()),
                        Err(err) => {
                            error!("Can not write {}. {:#}", filename, err);
                            // render causes as well
                            let mut err = &err as &dyn std::error::Error;
                            while let Some(next_err) = err.source() {
                                error!("caused by: {:#}", next_err);
                                err = next_err;
                            }
                        }
                    }
                }
            }
        }
        posts.sort_by(|a, b| b.date.cmp(&a.date));
    }
    posts
}

async fn post_with_mastodon(configuration: &Configuration, episode: &Episode,
        mastodon: &Mastodon) {
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    let ctx = context! {
        url => url,
        podcast => configuration.get_podcast(),
        post => episode.get_post(),
    };
    let template = ENV.get_template("mastodon.html").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            debug!("{}", content);
            mastodon.post(&content);
        }
        Err(err) => {
            error!("Algo no ha funcionado correctamente. {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn post_with_telegram(configuration: &Configuration, episode: &Episode,
        telegram: &Telegram) {
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    let post = episode.get_post();
    let audio = format!(
        "https://archive.org/download/{}/{}",
        &post.identifier, &post.filename
    );
    let ctx = context! {
        url => url,
        podcast => configuration.get_podcast(),
        audio => audio,
        post => episode.get_post(),
    };
    let template = ENV.get_template("telegram.html").unwrap();
    match template.render(ctx) {
        Ok(caption) => {
            info!("Caption: {caption}");
            telegram.send_audio(&audio, &caption);
        }
        Err(err) => {
            error!("Algo no ha funcionado correctamente. {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_feed(configuration: &Configuration, posts: &[Post]) {
    debug!("generate_feed");
    let public = if configuration.get_podcast().base_url.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_podcast().base_url
        )
    };
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    let ctx = context! {
        url => url,
        podcast => configuration.get_podcast(),
        posts => posts,
    };
    let template = ENV.get_template("feed.xml").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            write_post(
                &public,
                "",
                Some(&configuration.get_podcast().feed_url),
                &content,
            )
            .await;
            debug!("write feed");
        }
        Err(err) => {
            error!("Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_stats(configuration: &Configuration, posts: &Vec<Post>, pages: &Vec<Post>) {
    debug!("generate_stats");
    let public = if configuration.get_podcast().base_url.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_podcast().base_url
        )
    };
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    let ctx = context! {
        url => url,
        podcast => configuration.get_podcast(),
        posts => posts,
        pages => pages,
    };
    let template = ENV.get_template("statistics.html").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            debug!("{}", content);
            create_dir(&format!("{}/{}", public, "statistics")).await;
            write_post(&public, "statistics", None, &content).await;
        }
        Err(err) => {
            error!("Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_index(configuration: &Configuration, posts: &Vec<Post>,
    pages: &Vec<Post>) {
    debug!("generate_index");
    let public = if configuration.get_podcast().base_url.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_podcast().base_url
        )
    };
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    let ctx = context! {
        url => url,
        podcast => configuration.get_podcast(),
        posts => posts,
        pages => pages,
    };
    let template = ENV.get_template("index.html").unwrap();
    match template.render(ctx) {
        Ok(content) => {
            debug!("{}", content);
            write_post(&public, "", None, &content).await;
        }
        Err(err) => {
            error!("Could not render template: {:#}", err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        }
    }
}

async fn generate_pages(configuration: &Configuration, pages: &Vec<Post>) {
    debug!("generate_pages");
    let public = if configuration.get_podcast().base_url.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_podcast().base_url
        )
    };
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    for page in pages {
        debug!("Write page: {:?}", page);
        let ctx = context!(
            url => url,
            podcast => configuration.get_podcast(),
            page => page,
        );
        let template = ENV.get_template("page.html").unwrap();
        match template.render(ctx) {
            Ok(content) => {
                debug!("{}", &content);
                debug!("Page: {:?}", &page);
                create_dir(&format!("{}/{}", public, &page.slug)).await;
                write_post(&public, &page.slug, None, &content).await
            }
            Err(err) => {
                error!("Could not render template: {:#}", err);
                // render causes as well
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
            }
        }

    }

}

async fn generate_html(configuration: &Configuration, posts: &[Post],
        pages: &[Post]) {
    debug!("generate_html");
    let public = if configuration.get_podcast().base_url.is_empty() {
        configuration.get_public().to_owned()
    } else {
        format!(
            "{}/{}",
            configuration.get_public(),
            configuration.get_podcast().base_url
        )
    };
    let url = if configuration.get_podcast().base_url.is_empty() {
        "".to_string()
    } else if configuration.get_podcast().base_url.starts_with('/') {
        configuration.get_podcast().base_url.to_owned()
    } else {
        format!("/{}", configuration.get_podcast().base_url)
    };
    for post in posts {
        debug!("Write post: {:?}", post);
        let ctx = context!(
            url => url,
            podcast => configuration.get_podcast(),
            post => post,
            pages => pages,
        );
        let template = ENV.get_template("post.html").unwrap();
        match template.render(ctx) {
            Ok(content) => {
                debug!("{}", &content);
                debug!("Post: {:?}", &post);
                create_dir(&format!("{}/{}", public, &post.slug)).await;
                write_post(&public, &post.slug, None, &content).await
            }
            Err(err) => {
                error!("Could not render template: {:#}", err);
                // render causes as well
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
            }
        }
    }
}

async fn update(configuration: &Configuration) {
    debug!("update");
    let mastodon_client = get_mastodon_client();
    let telegram_client = get_telegram_client();
    let mut new_docs = Vec::new();
    let iaclient = configuration.get_iaclient();
    let docs = iaclient.get_all_docs();
    for doc in docs {
        if doc.exists().await {
            debug!("Doc {} exists", doc.get_identifier());
            debug!("Doc: {:?}", &doc);
            let filename = doc.get_post_filename();
            //BUG: Esto hay que revisar
            match Episode::new(&filename).await {
                Ok(ref mut episode) => {
                    if episode.get_downloads() != doc.get_downloads()
                    {
                        episode.set_downloads(doc.get_downloads());
                        match episode.save().await {
                            Ok(_) => info!("Episode {} saved", episode.get_identifier()),
                            Err(err) => {
                                error!("1 Can not save episode {}. {:#}", episode.get_identifier(), err);
                                // render causes as well
                                let mut err = &err as &dyn std::error::Error;
                                while let Some(next_err) = err.source() {
                                    error!("caused by: {:#}", next_err);
                                    err = next_err;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("Can not create episode. {:#}", err);
                    // render causes as well
                    let mut err = &err as &dyn std::error::Error;
                    while let Some(next_err) = err.source() {
                        error!("caused by: {:#}", next_err);
                        err = next_err;
                    }
                }
            }
        } else {
            new_docs.push(doc);
        }
    }
    for mut doc in new_docs {
        if let Err(e) = doc.complete() {
            error!("Can' complete doc: {e}");
        }else{
            let episode: Episode = doc.into();
            match episode.save().await {
                Ok(_) => {
                    match &telegram_client {
                        Some(client) => {
                            post_with_telegram(configuration, &episode, client).await;
                        }
                        None => {}
                    };
                    match &mastodon_client {
                        Some(client) => {
                            post_with_mastodon(configuration, &episode, client).await;
                        }
                        None => {}
                    }
                    info!("Episode {} saved", episode.get_identifier());
                }
                Err(err) => {
                    error!("2 Can not save episode {}. {:#}", episode.get_identifier(), err);
                    // render causes as well
                    let mut err = &err as &dyn std::error::Error;
                    while let Some(next_err) = err.source() {
                        error!("caused by: {:#}", next_err);
                        err = next_err;
                    }
                }
            }
        }
    }
}

fn clean_path(path: &str) -> &str {
    let path = if path.starts_with('/') {
        path.to_string().remove(0);
        path
    } else {
        path
    };
    if path.ends_with('/') {
        path.to_string().pop();
        path
    } else {
        path
    }
}

async fn write_post(base: &str, endpoint: &str, filename: Option<&str>, content: &str) {
    debug!(
        "write_post. Base: {base}. Endpoint {endpoint}. Filename: {:?}",
        filename
    );
    let base = clean_path(base);
    let endpoint = clean_path(endpoint);
    let filename = filename.unwrap_or("index.html");
    let output = if endpoint.is_empty() {
        format!("{}/{}", base, filename)
    } else {
        format!("{}/{}/{}", base, endpoint, filename)
    };
    match tokio::fs::write(&output, content).await {
        Ok(_) => debug!("post {} created", &output),
        Err(err) => {
            error!("Can not create post {}. {:#}", &output, err);
            // render causes as well
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}

async fn copy_all_files(from_dir: &str, to_dir: &str) {
    debug!("Going to copy from {} to {}", from_dir, to_dir);
    let mut episodes_dir = tokio::fs::read_dir(from_dir).await.unwrap();
    while let Some(file) = episodes_dir.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_file() {
            let filename = file.file_name().to_str().unwrap().to_string();
            let input_file = format!("{}/{}", from_dir, filename);
            let output_file = format!("{}/{}", to_dir, filename);
            copy_file(&input_file, &output_file).await;
        }
    }
}

async fn create_dir(output: &str) {
    debug!("Going to create : {}", &output);
    let exists = match tokio::fs::metadata(&output).await {
        Ok(metadata) => {
            debug!("Output dir {} exists", &output);
            metadata.is_dir()
        }
        Err(err) => {
            debug!("Can not get metadata for dir {}, {:#}", &output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                debug!("caused by: {:#}", next_err);
                err = next_err;
            }
            false
        }
    };
    if exists {
        match tokio::fs::remove_dir_all(&output).await {
            Ok(_) => debug!("Directory {} removed", output),
            Err(err) => {
                error!("Cant delete directory {}, {:#}", &output, err);
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
                std::process::exit(1);
            }
        }
    }
    match tokio::fs::create_dir_all(&output).await {
        Ok(_) => debug!("Directory {} created", output),
        Err(err) => {
            error!("Cant create directory {}, {:#}", &output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}

pub async fn copy_file(from: &str, to: &str) {
    match tokio::fs::copy(from, to).await {
        Ok(_) => debug!("Copied from {} to {}", from, to),
        Err(err) => {
            error!("Cant copy from {} to {}. {:#}", from, to, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}

pub async fn create_public(configuration: &Configuration) {
    debug!("create_public");
    let output = configuration.get_public();
    debug!("Output dir: {}", &output);
    let exists = match tokio::fs::metadata(output).await {
        Ok(metadata) => {
            debug!("Output dir {} exists", &output);
            metadata.is_dir()
        }
        Err(err) => {
            debug!("Output dir {}, {:#}", &output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            false
        }
    };
    if exists {
        match tokio::fs::remove_dir_all(output).await {
            Ok(_) => debug!("Directory {} removed", output),
            Err(err) => {
                error!("Cant delete directory {}, {}", output, err);
                let mut err = &err as &dyn std::error::Error;
                while let Some(next_err) = err.source() {
                    error!("caused by: {:#}", next_err);
                    err = next_err;
                }
                std::process::exit(1);
            }
        }
    }
    match tokio::fs::create_dir_all(output).await {
        Ok(_) => debug!("Directory {} created", output),
        Err(err) => {
            error!("Cant create directory {}, {}", output, err);
            let mut err = &err as &dyn std::error::Error;
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
            std::process::exit(1);
        }
    }
}
