# 正多角形プロット

このプロジェクトは正多角形をプロットするRustアプリケーションです。

## 特徴

- 指定された辺の数の正多角形をプロット
- 多角形のサイズと色をカスタマイズ
- プロットを画像ファイルとして保存

## インストール

1. Rustがインストールされていることを確認してください。インストールされていない場合は、[rust-lang.org](https://www.rust-lang.org/)からインストールできます。
2. リポジトリをクローンします:
    ```sh
    git clone https://github.com/yourusername/regular_polygon_plot.git
    ```
3. プロジェクトディレクトリに移動します:
    ```sh
    cd regular_polygon_plot
    ```
4. プロジェクトをビルドします:
    ```sh
    cargo build --release
    ```

## 使用方法

以下のコマンドでアプリケーションを実行します:
```sh
cargo run --release -- <number_of_sides> <size> <color>
```
例:
```sh
cargo run --release -- 5 100 red
```

## コントリビュート

コントリビュートは歓迎します！イシューをオープンするか、プルリクエストを送信してください。

## ライセンス

このプロジェクトはMITライセンスの下でライセンスされています。詳細は[LICENSE](LICENSE)ファイルを参照してください。