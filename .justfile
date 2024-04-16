user    := "atareao"
name    := `basename ${PWD}`
version := `git tag -l  | tail -n1`

default:
    @just --list

build:
    echo {{version}}
    echo {{name}}
    docker build -t {{user}}/{{name}}:{{version}} \
                 -t {{user}}/{{name}}:latest \
                 .

tag:
    docker tag {{user}}/{{name}}:{{version}} {{user}}/{{name}}:latest

push:
    docker push {{user}}/{{name}} --all-tags

run:
    docker run --rm \
               --init \
               --name aopodcast \
               --volume $PWD/config.yml:/app/config.yml \
               --volume $PWD/data:/app/data \
               --volume $PWD/assets:/app/assets \
               --volume $PWD/templates:/app/templates \
               --volume $PWD/episodes:/app/episodes \
               --volume $PWD/public:/app/public \
               --volume $PWD/posts:/app/posts \
               --volume $PWD/pages:/app/pages \
               {{user}}/{{name}}

test:
    echo {{version}}
    echo {{name}}
    docker build -t {{user}}/{{name}}:test \
                 .
    docker push {{user}}/{{name}}:test

