# Classification

This provides an easy REST API for text classification.

## Prerequisites

Ensure you have Rust and Cargo installed. If not, follow the instructions [here](https://www.rust-lang.org/learn/get-started).

## Setting Up

1. Clone this repository:
    ```shell
    git clone https://github.com/benjaminjacobberg/classification/
    ```

2. Navigate to the project directory:
    ```shell
    cd classification/classification-service
    ```

3. Run:
    ```shell
    cargo run
    ```

The API server will start at `http://localhost:8080`.

## Usage

In your browser, you can use the UI to try out the API at `http://localhost:8080`.

To classify a piece of text using the API directly, send a POST request to the `/api/classify` endpoint with your text in the request body. Here's a sample `curl` command:

```shell
curl --request POST \
  --url http://localhost:8080/api/classify \
  --header 'accept: application/json' \
  --header 'content-type: application/json' \
  --data '{"text": "Your lengthy text here...", "categories": ["item1", "item2", "item3"]}'
```

## Dependencies

- [actix](https://github.com/actix/actix)
- [actix-web](https://github.com/actix/actix-web)
- [tokenizers](https://github.com/huggingface/tokenizers)
- [text-splitter](https://github.com/benbrandt/text-splitter)
- [rust-bert](https://github.com/guillaume-be/rust-bert)

## License

[MIT License](LICENSE)