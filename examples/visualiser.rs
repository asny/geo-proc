
use dust::*;
use dust::objects::*;
use dust::window::{event::*, Window};
use geo_proc::*;
use tri_mesh::mesh::Mesh;

fn main() {
    let mut window = Window::new_default("Geometry visualiser").unwrap();
    let (width, height) = window.framebuffer_size();
    let gl = window.gl();

    // Renderer
    let renderer = DeferredPipeline::new(&gl, width, height, true, vec4(0.8, 0.8, 0.8, 1.0)).unwrap();
    let mirror_renderer = DeferredPipeline::new(&gl, width/2, height/2, true, vec4(0.8, 0.8, 0.8, 1.0)).unwrap();

    // Camera
    let mut camera = camera::PerspectiveCamera::new(dust::vec3(5.0, 3.0, 5.0), dust::vec3(0.0, 1.0, 0.0),
                                                    dust::vec3(0.0, 1.0, 0.0),degrees(45.0), width as f32 / height as f32, 0.1, 1000.0);

    // Objects
    let mut objects = Objects::new(&gl);
    objects.add(include_str!("../original_mesh1.obj").to_string());
    objects.add(include_str!("../original_mesh2.obj").to_string());
    objects.add(include_str!("../mesh1.obj").to_string());
    objects.add(include_str!("../mesh2.obj").to_string());

    let plane_positions: Vec<f32> = vec![
        -1.0, 0.0, -1.0,
        1.0, 0.0, -1.0,
        1.0, 0.0, 1.0,
        -1.0, 0.0, 1.0
    ];
    let plane_normals: Vec<f32> = vec![
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0
    ];
    let plane_indices: Vec<u32> = vec![
        0, 2, 1,
        0, 3, 2,
    ];
    let mut plane = crate::objects::ShadedMesh::new(&gl, &plane_indices, &att!["position" => (plane_positions, 3), "normal" => (plane_normals, 3)]).unwrap();
    plane.diffuse_intensity = 0.2;
    plane.specular_intensity = 0.4;
    plane.specular_power = 20.0;

    let mut ambient_light = light::AmbientLight::new();
    ambient_light.base.intensity = 0.2;

    let mut light1 = light::SpotLight::new(vec3(5.0, 5.0, 5.0), vec3(-1.0, -1.0, -1.0));
    light1.enable_shadows(&gl, 20.0).unwrap();
    light1.base.intensity = 0.5;

    let mut light2 = light::SpotLight::new(vec3(-5.0, 5.0, 5.0), vec3(1.0, -1.0, -1.0));
    light2.enable_shadows(&gl, 20.0).unwrap();
    light2.base.intensity = 0.5;

    let mut light3 = light::SpotLight::new(vec3(-5.0, 5.0, -5.0), vec3(1.0, -1.0, 1.0));
    light3.enable_shadows(&gl, 20.0).unwrap();
    light3.base.intensity = 0.5;

    let mut light4 = light::SpotLight::new(vec3(5.0, 5.0, -5.0), vec3(-1.0, -1.0, 1.0));
    light4.enable_shadows(&gl, 20.0).unwrap();
    light4.base.intensity = 0.5;

    // Mirror
    let mirror_program = program::Program::from_source(&gl, include_str!("copy.vert"),
                                                                 include_str!("mirror.frag")).unwrap();

    let mut camera_handler = camerahandler::CameraHandler::new(camerahandler::CameraState::SPHERICAL);

    // main loop
    let mut j = 0;
    window.render_loop(move |events, _elapsed_time|
    {
        for event in events {
            handle_camera_events(event, &mut camera_handler, &mut camera);

            match event {
                Event::Key { state, kind } => {
                    if kind == "Key1" && *state == State::Pressed
                    {
                        objects.toggle(0);
                    }
                    else if kind == "Key2" && *state == State::Pressed
                    {
                        objects.toggle(1);
                    }
                    else if kind == "Key3" && *state == State::Pressed
                    {
                        objects.toggle(2);
                    }
                    else if kind == "Key4" && *state == State::Pressed
                    {
                        objects.toggle(3);
                    }
                },
                _ => {}
            }
        }

        // Draw
        let render_scene = |camera: &Camera| {
            objects.render(camera);
        };

        // Shadow pass
        light1.shadow_cast_begin().unwrap();
        render_scene(light1.shadow_camera().unwrap());

        light2.shadow_cast_begin().unwrap();
        render_scene(light2.shadow_camera().unwrap());

        light3.shadow_cast_begin().unwrap();
        render_scene(light3.shadow_camera().unwrap());

        light4.shadow_cast_begin().unwrap();
        render_scene(light4.shadow_camera().unwrap());

        // Mirror pass
        camera.mirror_in_xz_plane();

        // Mirror pass (Geometry pass)
        mirror_renderer.geometry_pass_begin().unwrap();
        render_scene(&camera);

        // Mirror pass (Light pass)
        mirror_renderer.light_pass_begin(&camera).unwrap();
        mirror_renderer.shine_ambient_light(&ambient_light).unwrap();
        mirror_renderer.shine_spot_light(&light1).unwrap();
        mirror_renderer.shine_spot_light(&light2).unwrap();
        mirror_renderer.shine_spot_light(&light3).unwrap();
        mirror_renderer.shine_spot_light(&light4).unwrap();

        camera.mirror_in_xz_plane();

        // Geometry pass
        renderer.geometry_pass_begin().unwrap();
        render_scene(&camera);
        plane.render(&Mat4::from_scale(100.0), &camera);

        // Light pass
        renderer.light_pass_begin(&camera).unwrap();
        renderer.shine_ambient_light(&ambient_light).unwrap();
        renderer.shine_spot_light(&light1).unwrap();
        renderer.shine_spot_light(&light2).unwrap();
        renderer.shine_spot_light(&light3).unwrap();
        renderer.shine_spot_light(&light4).unwrap();

        // Blend with the result of the mirror pass
        state::blend(&gl,state::BlendType::SRC_ALPHA__ONE_MINUS_SRC_ALPHA);
        state::depth_write(&gl,false);
        state::depth_test(&gl, state::DepthTestType::NONE);
        state::cull(&gl,state::CullType::BACK);

        mirror_renderer.light_pass_color_texture().unwrap().bind(0);
        mirror_program.add_uniform_int("colorMap", &0).unwrap();
        full_screen_quad::render(&gl, &mirror_program);

        //renderer.save_screenshot(&format!("ss/image{}.png", j)).unwrap();
        j = j+1;

        renderer.copy_to_screen().unwrap();
    }).unwrap();
}

pub fn handle_camera_events(event: &Event, camera_handler: &mut dust::camerahandler::CameraHandler, camera: &mut Camera)
{
    match event {
        Event::Key {state, kind} => {
            if kind == "Tab" && *state == State::Pressed
            {
                camera_handler.next_state();
            }
        },
        Event::MouseClick {state, button, ..} => {
            if *button == MouseButton::Left
            {
                if *state == State::Pressed { camera_handler.start_rotation(); }
                else { camera_handler.end_rotation() }
            }
        },
        Event::MouseMotion {delta} => {
            camera_handler.rotate(camera, delta.0 as f32, delta.1 as f32);
        },
        Event::MouseWheel {delta} => {
            camera_handler.zoom(camera, *delta as f32);
        }
    }
}

struct Objects
{
    models: Vec<(bool, dust::objects::ShadedMesh, dust::objects::Wireframe)>,
    gl: gl::Gl
}

impl Objects
{
    fn new(gl: &gl::Gl) -> Objects
    {
        Objects {models: Vec::new(), gl: gl.clone()}
    }

    fn add(&mut self, model_source: String)
    {
        let mut model = ShadedMesh::new_from_obj_source(&self.gl, model_source.clone()).unwrap();
        model.color = vec3(0.5, 0.5, 0.5);

        let mut wireframe = Wireframe::new_from_obj_source(&self.gl, model_source, 0.02, &vec3(0.0, 2.0, 0.0));
        wireframe.set_parameters(0.8, 0.2, 5.0);
        wireframe.set_color(&vec3(1.0, 0.5, 0.5));

        self.models.push((false, model, wireframe));
    }

    fn toggle(&mut self, i: usize)
    {
        if i < 0 || i > self.models.len() { return; }
        self.models[i].0 = !self.models[i].0;
    }

    fn render(&self, camera: &Camera)
    {
        let model_matrix = Mat4::from_translation(vec3(0.0, 2.0, 0.0));
        for (enabled, model, wireframe) in self.models.iter() {
            if *enabled {
                model.render(&model_matrix, camera);
                wireframe.render(camera);
            }
        }
    }
}
