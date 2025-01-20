# readme

An example of implementing sprite blend mode effect based on wgpu.

ps: I only implemented hard light effect, and so on for other effects.

![showcase.png](showcase.png)

其中 simple_draw 主要是为什么试验是否能更加高效的实现 grab pass 功能，目前看不行，因为一张 texture 在一个 render pass
中没法同时作为 color target 和 bind group，离屏渲染并不能带来更高效的实现方案。

尝试简单的实现了一版高斯模糊效果，如果要迁移到 bevy 需要注意，如果可以调整模糊半径，就意味着需要准备多个不同尺寸的高斯掩膜。同时，高斯掩膜的应该是基于
world 坐标的，感觉计算逻辑还需要考虑镜头的放大缩小。

参考文档：

1. https://en.wikipedia.org/wiki/Gaussian_blur
2. https://www.zhihu.com/question/54918332
3. https://www.ruanyifeng.com/blog/2012/11/gaussian_blur.html