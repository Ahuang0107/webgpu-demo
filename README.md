# readme

## 一些功能实现的笔记

1. 离屏渲染并不能更加高效的实现 grab pass 功能，目前看不行，因为一张 texture 在一个 render pass
   中没法同时作为 color target 和 bind group 所以无论如何一定会有一个将当前渲染目标的纹理 copy 出来作为一张单独的纹理再传给
   shader 的过程
2. 尝试简单的实现了一版高斯模糊效果，需要注意，如果可以调整模糊半径，就意味着需要准备多个不同尺寸的高斯掩膜。同时，高斯掩膜的应该是基于
   world 坐标的，感觉计算逻辑还需要考虑镜头的放大缩小。
   参考文档：

    1. https://en.wikipedia.org/wiki/Gaussian_blur
    2. https://www.zhihu.com/question/54918332
    3. https://www.ruanyifeng.com/blog/2012/11/gaussian_blur.html

## 打包 WASM

```shell
cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web --out-dir dist --out-name webgpu_demo_lib target/wasm32-unknown-unknown/wasm-release/webgpu_demo_lib.wasm
http-server -p 8080
```