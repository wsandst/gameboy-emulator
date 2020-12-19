mod renderer;

fn main() {
    let mut renderer = renderer::Renderer::new();

    // Main game loop
    loop 
    {  
        renderer.render();
        let exit = renderer.input();
        if exit {
            break;
        }
    }
}
