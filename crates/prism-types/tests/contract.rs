//! `prism-types` 的公开契约测试。
//!
//! 只经 crate 的公开 API 断言，不引用内部模块路径——这个 crate 的全部价值就是
//! 它对外暴露的形状（D-09），所以测试也从外部视角写。

use std::sync::Arc;

use prism_types::{
    CommentSink, EngineEvent, FeedbackItem, FeedbackSource, Receipt, SearchHit, ServiceError,
};

const KNOWN_PROJECT: &str = "01K0PRISMPROJECT000000000A";
const UNKNOWN_PROJECT: &str = "01K0PRISMPROJECT000000000B";

/// 假引擎：证明两个 trait 可以被同一个具体类型实现并装进 `Arc<dyn …>`。
struct FakeEngine;

impl FeedbackSource for FakeEngine {
    fn list_feedback(&self, project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
        if project_id != KNOWN_PROJECT {
            return Err(ServiceError::NotFound);
        }
        Ok(vec![FeedbackItem {
            id: "fb-1".to_string(),
            doc_id: "doc-1".to_string(),
            body: "锚点在这一段漂移了".to_string(),
        }])
    }
}

impl CommentSink for FakeEngine {
    fn record_receipt(&self, receipt: Receipt) -> Result<(), ServiceError> {
        if receipt.comment_id.is_empty() {
            return Err(ServiceError::Invalid(
                "comment id must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[test]
fn service_traits_are_object_safe_behind_arc() {
    let engine = Arc::new(FakeEngine);
    let source: Arc<dyn FeedbackSource> = engine.clone();
    let sink: Arc<dyn CommentSink> = engine;

    let items = source
        .list_feedback(KNOWN_PROJECT)
        .expect("known project resolves");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].doc_id, "doc-1");

    sink.record_receipt(Receipt {
        comment_id: "c-1".to_string(),
        status: "applied".to_string(),
    })
    .expect("receipt is accepted");
}

#[test]
fn service_error_display_does_not_echo_caller_arguments() {
    let source: Arc<dyn FeedbackSource> = Arc::new(FakeEngine);
    let err = source
        .list_feedback(UNKNOWN_PROJECT)
        .expect_err("unknown project is rejected");

    let shown = err.to_string();
    assert!(
        !shown.contains(UNKNOWN_PROJECT),
        "ServiceError leaked the caller-supplied identifier: {shown}"
    );

    // 同时锁死它实现了 std::error::Error（供 `?` 与 source 链使用）。
    let as_std: &dyn std::error::Error = &err;
    assert!(!as_std.to_string().is_empty());
}

#[test]
fn doc_changed_serialises_with_camel_case_tag_and_fields() {
    let json = serde_json::to_string(&EngineEvent::DocChanged {
        project_id: KNOWN_PROJECT.to_string(),
        doc_id: "doc-1".to_string(),
    })
    .expect("EngineEvent serialises");

    assert!(json.contains("\"kind\""), "missing tag field: {json}");
    assert!(json.contains("\"projectId\""), "not camelCase: {json}");
    assert!(json.contains("\"docId\""), "not camelCase: {json}");
    assert!(!json.contains("project_id"), "snake_case leaked: {json}");
}

#[test]
fn resync_serialises_as_a_bare_tagged_object() {
    let json = serde_json::to_string(&EngineEvent::Resync).expect("Resync serialises");

    assert_eq!(json, r#"{"kind":"resync"}"#);
    assert!(!json.contains("projectId"), "Resync must carry no payload");
}

#[test]
fn inbox_updated_carries_only_an_id_and_a_count() {
    let json = serde_json::to_string(&EngineEvent::InboxUpdated {
        project_id: KNOWN_PROJECT.to_string(),
        unread: 7,
    })
    .expect("InboxUpdated serialises");

    assert!(json.contains("\"unread\":7"), "unexpected payload: {json}");
    assert!(json.contains("\"projectId\""), "not camelCase: {json}");
}

#[test]
fn engine_event_is_clonable_and_round_trips() {
    let original = EngineEvent::DocChanged {
        project_id: KNOWN_PROJECT.to_string(),
        doc_id: "doc-1".to_string(),
    };
    // Clone 是总线契约的一部分：broadcast::Sender<EngineEvent> 要求它。
    let json = serde_json::to_string(&original.clone()).expect("serialises");
    let back: EngineEvent = serde_json::from_str(&json).expect("deserialises");

    match back {
        EngineEvent::DocChanged { project_id, doc_id } => {
            assert_eq!(project_id, KNOWN_PROJECT);
            assert_eq!(doc_id, "doc-1");
        }
        other => panic!("round-trip changed the variant: {other:?}"),
    }
}

#[test]
fn search_hit_serialises_with_camel_case_fields() {
    let json = serde_json::to_string(&SearchHit {
        doc_id: "doc-1".to_string(),
        title: None,
        rel_path: "docs/api.md".to_string(),
    })
    .expect("SearchHit serialises");

    assert!(json.contains("\"docId\""), "not camelCase: {json}");
    assert!(json.contains("\"relPath\""), "not camelCase: {json}");
    assert!(
        json.contains("\"title\":null"),
        "unexpected payload: {json}"
    );
}
