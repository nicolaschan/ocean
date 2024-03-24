use std::cell::RefMut;

use crate::app::state::TerminalState;

use super::vertex::VERTICES;

pub fn render_on_device(
    font: &fontdue::Font,
    state: &TerminalState,
    mut encoder: RefMut<wgpu::CommandEncoder>,
    texture_view: &wgpu::TextureView,
    bind_group: &wgpu::BindGroup,
    render_pipeline: &wgpu::RenderPipeline,
    vertex_buffer: &wgpu::Buffer,
) {
    let mut render_pass = nannou::wgpu::RenderPassBuilder::new()
        .color_attachment(texture_view, |color| color)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &bind_group, &[]);
    render_pass.set_pipeline(&render_pipeline);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

    let vertex_range = 0..VERTICES.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}
