# readme

An example of implementing sprite blend mode effect based on wgpu.

ps: I only implemented hard light effect, and so on for other effects.

![showcase.png](showcase.png)

其中 simple_draw 主要是为什么试验是否能更加高效的实现 grab pass 功能，目前看不行，因为一张 texture 在一个 render pass
中没法同时作为 color target 和 bind group，离屏渲染并不能带来更高效的实现方案。