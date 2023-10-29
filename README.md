# Synopsis

This provides an easy REST API for text summarization.

## Prerequisites

Ensure you have Rust and Cargo installed. If not, follow the instructions [here](https://www.rust-lang.org/learn/get-started).

## Setting Up

1. Clone this repository:
    ```shell
    git clone https://github.com/benjaminjacobberg/synopsis/
    ```

2. Navigate to the project directory:
    ```shell
    cd synopis/synopsis-service
    ```

3. Run:
    ```shell
    cargo run
    ```

The API server will start at `http://localhost:8080`.

## Usage

To summarize a piece of text, send a POST request to the `/api/summarize` endpoint with your text in the request body. Here's a sample `curl` command:

```shell
curl --request POST \
  --url http://localhost:8080/api/summarize \
  --header 'accept: application/json' \
  --header 'content-type: application/json' \
  --data '{"text": "Your lengthy text here...", "range": {"min": 50, "max": 100}}'
```

## Dependencies

- [actix](https://github.com/actix/actix)
- [actix-web](https://github.com/actix/actix-web)
- [tokenizers](https://github.com/huggingface/tokenizers)
- [text-splitter](https://github.com/benbrandt/text-splitter)
- [rust-bert](https://github.com/guillaume-be/rust-bert)

## License

[MIT License](LICENSE)