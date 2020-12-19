mod renderer;

fn main() {
    let mut renderer = renderer::Renderer::new();

    let mut buffer : [u8; 160*144*3] = [0; 160*144*3];

    for i in 0..(buffer.len()/3) {
        buffer[i*3+0] = 0;
        buffer[i*3+1] = 255;
        buffer[i*3+2] = 255;
    }

    println!("S {}", buffer.len());
    renderer.set_screen_buffer(&mut buffer);

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
