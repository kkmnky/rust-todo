# Todoアプリ（React+Rust）

## 概要
Rustを勉強するためのTodoアプリです。

このTodoアプリは、書籍[Webアプリ開発で学ぶ よくわかるRustプログラミング入門](https://www.amazon.co.jp/dp/4798067318)で実装されている[Todoアプリ](https://github.com/AkifumiSato/learn-rust-with-web-application)をベースに作成されています。自己学習のため色々と機能追加や変更をする予定です。

## 動作確認時のバージョン

- Docker 25.0.2
- rust 1.81.0
- node 18.20.1

## ローカル起動

api/Makefileのsqlxのパスは各自修正ください

1. バックエンドをビルド
    ```bash
    cd api
    make build
    cd ..
    ```
2. データベース起動
    ```bash
    cd api
    make db
    cd ..
    ```
3. 別タブを開き、バックエンド起動
    ```bash
    cd api
    make dev
    cd ..
    ```
4. 別タブを開き、フロントエンド起動
    ```bash
    cd front
    npm run dev
    cd ..
    ```