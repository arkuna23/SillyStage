use futures_util::StreamExt;
use serde_json::json;
use ss_llm_api::{ChatRequest, LlmApi, LlmError, OpenAiClient, OpenAiConfig, ResponseFormat, Role};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn chat_uses_default_model_and_maps_response() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(json!({
        "model": "gpt-4o-mini",
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "hello from OpenAI"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 4,
            "total_tokens": 16
        }
    }));

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": "Say hello"
                }
            ]
        })))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = client_for_server(&server).expect("client should build");
    let response = client
        .chat(
            ChatRequest::builder()
                .user_message("Say hello")
                .build()
                .expect("request should build"),
        )
        .await
        .expect("chat should succeed");

    assert_eq!(response.message.role, Role::Assistant);
    assert_eq!(response.message.content, "hello from OpenAI");
    assert_eq!(response.model, "gpt-4o-mini");
    assert_eq!(response.finish_reason.as_deref(), Some("stop"));
    assert_eq!(response.usage.expect("usage").total_tokens, 16);
    assert_eq!(response.structured_output, None);
}

#[tokio::test]
async fn chat_request_model_overrides_default_model() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({
            "model": "gpt-4.1-mini"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "gpt-4.1-mini",
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "override ok"
                    },
                    "finish_reason": "stop"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = client_for_server(&server).expect("client should build");
    let response = client
        .chat(
            ChatRequest::builder()
                .user_message("Use another model")
                .model("gpt-4.1-mini")
                .build()
                .expect("request should build"),
        )
        .await
        .expect("chat should succeed");

    assert_eq!(response.model, "gpt-4.1-mini");
}

#[tokio::test]
async fn chat_stream_yields_chunks_and_terminal_event() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"model\":\"gpt-4o-mini\",\"choices\":[{\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
        "data: {\"model\":\"gpt-4o-mini\",\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":null}]}\n\n",
        "data: {\"model\":\"gpt-4o-mini\",\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n"
    );

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({
            "stream": true
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
        .mount(&server)
        .await;

    let client = client_for_server(&server).expect("client should build");
    let stream = client
        .chat_stream(
            ChatRequest::builder()
                .user_message("Stream hello")
                .build()
                .expect("request should build"),
        )
        .await
        .expect("stream should start");

    let chunks = stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("stream should parse");

    assert_eq!(chunks[0].delta, "Hello");
    assert_eq!(chunks[1].delta, " world");
    assert_eq!(chunks[2].finish_reason.as_deref(), Some("stop"));
    assert!(!chunks[2].done);
    assert!(chunks[3].done);
}

#[tokio::test]
async fn chat_maps_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "message": "invalid auth token"
            }
        })))
        .mount(&server)
        .await;

    let client = client_for_server(&server).expect("client should build");
    let error = client
        .chat(
            ChatRequest::builder()
                .user_message("hello")
                .build()
                .expect("request should build"),
        )
        .await
        .expect_err("chat should fail");

    assert!(matches!(error, LlmError::Authentication));
}

#[tokio::test]
async fn chat_with_json_object_populates_structured_output() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(json!({
        "model": "gpt-4o-mini",
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "{\"name\":\"Jane\",\"age\":54}"
                },
                "finish_reason": "stop"
            }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({
            "response_format": {
                "type": "json_object"
            }
        })))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = client_for_server(&server).expect("client should build");
    let response = client
        .chat(
            ChatRequest::builder()
                .user_message("Extract the person record")
                .response_format(ResponseFormat::JsonObject)
                .build()
                .expect("request should build"),
        )
        .await
        .expect("chat should succeed");

    assert_eq!(response.message.content, "{\"name\":\"Jane\",\"age\":54}");
    assert_eq!(
        response.structured_output,
        Some(json!({
            "name": "Jane",
            "age": 54
        }))
    );
}

#[tokio::test]
async fn chat_with_json_object_invalid_json_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "gpt-4o-mini",
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "definitely not json"
                    },
                    "finish_reason": "stop"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = client_for_server(&server).expect("client should build");
    let error = client
        .chat(
            ChatRequest::builder()
                .user_message("Return JSON")
                .response_format(ResponseFormat::JsonObject)
                .build()
                .expect("request should build"),
        )
        .await
        .expect_err("chat should fail");

    assert!(matches!(
        error,
        LlmError::StructuredOutputParse {
            message: _,
            raw_content: _
        }
    ));
}

#[tokio::test]
async fn chat_stream_rejects_json_object_requests() {
    let server = MockServer::start().await;
    let client = client_for_server(&server).expect("client should build");
    let result = client
        .chat_stream(
            ChatRequest::builder()
                .user_message("Stream JSON")
                .response_format(ResponseFormat::JsonObject)
                .build()
                .expect("request should build"),
        )
        .await;

    assert!(matches!(result, Err(LlmError::InvalidRequest(_))));
}

fn client_for_server(server: &MockServer) -> Result<OpenAiClient, LlmError> {
    let config = OpenAiConfig::builder()
        .api_key("test-key")
        .default_model("gpt-4o-mini")
        .base_url(format!("{}/", server.uri()))
        .build()?;

    OpenAiClient::new(config)
}
