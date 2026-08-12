use crate::material::Shader;
use crate::mesh::Mesh;
use crate::renderer::TriangleRenderer;
use crate::text_renderer::TextRenderer;
use std::ops::DerefMut;
use std::sync::Arc;
use winit::window::Window;

/// Base trait every game implements. The engine hands the window to the game,
/// and drives the frame loop through this trait.
///
/// The `DerefMut<Target = BaseGame>` supertrait removes the old `base_mut`
/// boilerplate: your game embeds a `BaseGame` (typically a `base` field) and
/// implements `Deref`/`DerefMut` for it, so every `BaseGame` method —
/// `add_mesh`, `uniforms`, `render`, ... — is callable straight on your
/// game object.
///
/// The game owns its HUD text as plain `TextObject` fields and hands the
/// base a snapshot each frame via `hud_texts` — the base draws them on top
/// of the meshes.
pub trait Game: DerefMut<Target = BaseGame> {
    /// Create the game state. The game builds its own renderer inside here.
    fn init(window: Arc<Window>) -> Self;

    /// Called once after init — spawn your text/objects here. Optional.
    fn start(&mut self) {}

    /// Called every frame with the time delta in seconds. Pure logic — the
    /// base handles rendering automatically right after this.
    fn update(&mut self, _dt: f32) {}

    /// Snapshot of the game's HUD text, handed to the base every frame for
    /// drawing. Games with no text can leave the default empty vec.
    fn hud_texts(&self) -> Vec<TextObject> {
        Vec::new()
    }

    /// Called when the window is resized. Default forwards to the base GPU
    /// surface resize.
    fn resize(&mut self, width: u32, height: u32) {
        let base: &mut BaseGame = self;
        base.resize(width, height);
    }

    /// Renders one frame — hidden in the base. Grabs the game's HUD text
    /// snapshot and hands it to `BaseGame::render`, which draws the meshes
    /// and the HUD text on top.
    fn render(&mut self) {
        let hud = self.hud_texts();
        let base: &mut BaseGame = self;
        base.render(&hud);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MyCustomVariables {
    pub current_time: f32,
    pub player_speed: f32,
    pub global_scale: f32,
    pub padding: f32,
}

/// A piece of HUD text. The game owns these as plain fields and mutates
/// them directly (`self.fps_text.text = ...`); each frame `Game::hud_texts`
/// hands the base a snapshot to draw on top of the meshes.
#[derive(Clone)]
pub struct TextObject {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub color: [f32; 4],
}

impl TextObject {
    /// `(x, y)` is the baseline of the first glyph; `scale` is the em-size in
    /// clip-space (1.0 fills the viewport height). Typical HUD text: 0.05..0.15.
    pub fn new(text: &str, x: f32, y: f32, scale: f32, color: [f32; 4]) -> Self {
        Self {
            text: text.to_string(),
            x,
            y,
            scale,
            color,
        }
    }
}

/// The base game class. Owns the renderer, the shared shader, the text
/// renderer, and the registered meshes. HUD text stays owned by the game
/// (plain `TextObject` fields) and is handed in as a snapshot each frame.
/// Your game struct embeds one of these and implements `Deref`/`DerefMut`
/// to it, so every method here is callable directly on the game.
pub struct BaseGame {
    pub renderer: TriangleRenderer,
    pub shader: Shader,
    pub text_renderer: TextRenderer,
    pub uniforms: MyCustomVariables,
    meshes: Vec<Mesh>,
}

impl BaseGame {
    /// Build everything the base needs: renderer, shared shader, and the TTF
    /// text renderer. Swap the font path to use a different .ttf/.otf.
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = TriangleRenderer::new(window);
        let shader_code = include_str!("shader.wgsl");
        let uniform_size = std::mem::size_of::<MyCustomVariables>() as u64;
        let shader = Shader::new(&renderer, shader_code, uniform_size);
        let text_renderer = TextRenderer::from_file(&renderer, "src/assets/fonts/arial.ttf");

        Self {
            renderer,
            shader,
            text_renderer,
            uniforms: MyCustomVariables {
                current_time: 0.0,
                player_speed: 1.0,
                global_scale: 1.0,
                padding: 0.0,
            },
            meshes: Vec::new(),
        }
    }

    /// Registers a (static) mesh with this base so it is rendered every frame.
    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    /// Render one frame. This is the whole pipeline:
    /// 1. Uploads `self.uniforms` to the GPU,
    /// 2. Clears + begins the scene pass,
    /// 3. Draws every registered mesh with the active mesh pipeline,
    /// 4. Draws the game's HUD text snapshot (`hud`) on top,
    /// 5. Submits the encoder and presents the frame.
    ///
    /// Games never call this directly — the engine drives it through the
    /// `Game::render` trait default, which feeds `hud_texts()` in here.
    pub fn render(&mut self, hud: &[TextObject]) {
        self.renderer.update_shader_buffer(&self.shader, &self.uniforms);
        self.renderer.render_scene(&self.shader, |ctx| {
            // 1. Registered meshes — drawn with the active mesh pipeline.
            for mesh in &self.meshes {
                mesh.draw(ctx);
            }

            // 2. HUD text on top — drawn from the game's snapshot.
            self.text_renderer.begin_frame();
            let text_refs: Vec<&TextObject> = hud.iter().collect();
            self.text_renderer.draw_objects(ctx, &text_refs);
        });
    }

    /// Resize the window surface. Games should forward their `resize` here
    /// instead of touching the renderer directly — or just rely on the
    /// `Game::resize` trait default.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }
}