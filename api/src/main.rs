mod handlers;
mod repositories;

use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    extract::Extension,
    routing::{delete, get, post},
    Router,
};
use dotenv::dotenv;
use handlers::{
    label::{all_label, create_label, delete_label},
    todo::{all_todo, create_todo, delete_todo, find_todo, update_todo},
};
use hyper::header::CONTENT_TYPE;
use repositories::{
    label::{LabelRepository, LabelRepositoryForDb},
    todo::{TodoRepository, TodoRepositoryForDb},
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer, Origin};

#[tokio::main]
async fn main() {
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    tracing::debug!("start connect database...");
    let pool = PgPool::connect(database_url)
        .await
        .unwrap_or_else(|_| panic!("failed connect database. url: {}", database_url));

    let app = create_app(
        TodoRepositoryForDb::new(pool.clone()),
        LabelRepositoryForDb::new(pool.clone()),
    );
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<Todo: TodoRepository, Label: LabelRepository>(
    todo_repository: Todo,
    label_repository: Label,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<Todo>).get(all_todo::<Todo>))
        .route(
            "/todos/:id",
            get(find_todo::<Todo>)
                .delete(delete_todo::<Todo>)
                .patch(update_todo::<Todo>),
        )
        .route(
            "/labels",
            post(create_label::<Label>).get(all_label::<Label>),
        )
        .route("/labels/:id", delete(delete_label::<Label>))
        .layer(Extension(Arc::new(todo_repository)))
        .layer(Extension(Arc::new(label_repository)))
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::exact("http://localhost:3001".parse().unwrap()))
                .allow_methods(Any)
                .allow_headers(vec![CONTENT_TYPE]),
        )
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repositories::{
        label::{test_utils::LabelRepositoryForMemory, Label},
        todo::{test_utils::TodoRepositoryForMemory, CreateTodo, TodoEntity},
    };
    use axum::{
        body::Body,
        http::{header, Method, Request},
        response::Response,
    };
    use hyper::StatusCode;
    use tower::ServiceExt;

    fn build_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_req_with_empty(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_todo(res: Response) -> TodoEntity {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let todo: TodoEntity = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("cannot convert Todo instance. body: {}", body));
        // .expect(&format!("cannot convert Todo instance. body: {}", body));
        todo
    }

    async fn res_to_label(res: Response) -> Label {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let label: Label = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("cannot convert Label instance. body: {}", body));
        label
    }

    fn label_fixture() -> (Vec<Label>, Vec<i32>) {
        let id = 999;
        (
            vec![Label {
                id,
                name: String::from("test label"),
            }],
            vec![id],
        )
    }

    #[tokio::test]
    async fn should_create_todo() {
        let (labels, _label_ids) = label_fixture();
        let expected = TodoEntity::new(1, "should_return_created_todo".to_string(), labels.clone());

        let req = build_req_with_json(
            "/todos",
            Method::POST,
            r#"{ "text": "should_return_created_todo", "labels": [999] }"#.to_string(),
        );
        let res = create_app(
            TodoRepositoryForMemory::new(labels),
            LabelRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_find_todo() {
        let (labels, label_ids) = label_fixture();
        let expected = TodoEntity::new(1, "should_find_todo".to_string(), labels.clone());

        let todo_repository = TodoRepositoryForMemory::new(labels.clone());
        todo_repository
            .create(CreateTodo::new("should_find_todo".to_string(), label_ids))
            .await
            .expect("failed find todo");
        let req = build_req_with_empty(Method::GET, "/todos/1");
        let res = create_app(todo_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todos() {
        let (labels, label_ids) = label_fixture();
        let expected = TodoEntity::new(1, "should_get_all_todos".to_string(), labels.clone());

        let todo_repository = TodoRepositoryForMemory::new(labels.clone());
        todo_repository
            .create(CreateTodo::new(
                "should_get_all_todos".to_string(),
                label_ids,
            ))
            .await
            .expect("failed get all todos");
        let req = build_req_with_empty(Method::GET, "/todos");
        let res = create_app(todo_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let todo: Vec<TodoEntity> = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("cannot convert Todo instance. body: {}", body));
        // .expect(&format!("cannot convert Todo instance. body: {}", body));
        assert_eq!(vec![expected], todo);
    }

    #[tokio::test]
    async fn should_update_todo() {
        let (labels, label_ids) = label_fixture();
        let expected = TodoEntity::new(1, "should_update_todo".to_string(), labels.clone());

        let todo_repository = TodoRepositoryForMemory::new(labels.clone());
        todo_repository
            .create(CreateTodo::new("before_update_todo".to_string(), label_ids))
            .await
            .expect("failed update todo");
        let req = build_req_with_json(
            "/todos/1",
            Method::PATCH,
            r#"{
                "id": 1,
                "text": "should_update_todo",
                "completed": false
            }"#
            .to_string(),
        );
        let res = create_app(todo_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_delete_todo() {
        let (labels, label_ids) = label_fixture();
        let todo_repository = TodoRepositoryForMemory::new(labels.clone());
        todo_repository
            .create(CreateTodo::new("should_delete_todo".to_string(), label_ids))
            .await
            .expect("failed delete todo");
        let req = build_req_with_empty(Method::DELETE, "/todos/1");
        let res = create_app(todo_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }

    #[tokio::test]
    async fn should_create_label() {
        let (labels, _label_ids) = label_fixture();
        let expected = Label::new(1, "should_return_created_label".to_string());

        let req = build_req_with_json(
            "/labels",
            Method::POST,
            r#"{ "name": "should_return_created_label" }"#.to_string(),
        );
        let res = create_app(
            TodoRepositoryForMemory::new(labels),
            LabelRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let label = res_to_label(res).await;
        assert_eq!(expected, label);
    }

    #[tokio::test]
    async fn should_get_all_labels() {
        let expected = Label::new(1, "should_get_all_labels".to_string());

        let label_repository = LabelRepositoryForMemory::new();
        let label = label_repository
            .create("should_get_all_labels".to_string())
            .await
            .expect("failed get all labels");
        let req = build_req_with_empty(Method::GET, "/labels");
        let res = create_app(TodoRepositoryForMemory::new(vec![label]), label_repository)
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let labels: Vec<Label> = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("cannot convert Label instance. body: {}", body));
        // .expect(&format!("cannot convert Todo instance. body: {}", body));
        assert_eq!(vec![expected], labels);
    }

    #[tokio::test]
    async fn should_delete_label() {
        let label_repository = LabelRepositoryForMemory::new();
        let label = label_repository
            .create("should_delete_label".to_string())
            .await
            .expect("failed delete label");
        let req = build_req_with_empty(Method::DELETE, "/labels/1");
        let res = create_app(TodoRepositoryForMemory::new(vec![label]), label_repository)
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }
}
