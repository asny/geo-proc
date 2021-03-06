
use dust::*;
use dust::objects::*;
use dust::window::{event::*, Window};
use geo_proc::*;
use tri_mesh::mesh::Mesh;
use tri_mesh::prelude::Vec3 as Vec3;
use tri_mesh::prelude::vec3 as vec3;
use tri_mesh::MeshBuilder;

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
    let (fire_mesh, result_mesh) = stitch("examples/stitching_data/", "examples/results/");
    let model = include_str!("stitching_data/model.obj").to_string();
    let objects = Objects::new(&gl, model, fire_mesh.parse_as_obj(), result_mesh.parse_as_obj());

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
    let mut i = 0;
    window.render_loop(move |events, _elapsed_time|
    {
        for event in events {
            handle_camera_events(event, &mut camera_handler, &mut camera);

            match event {
                Event::Key { state, kind } => {
                    if kind == "Q" && *state == State::Pressed
                    {
                        i = (i - 1).max(0);
                    }
                    if kind == "W" && *state == State::Pressed
                    {
                        i = (i + 1).min(3);
                    }
                },
                _ => {}
            }
        }

        // Draw
        let render_scene = |camera: &Camera| {
            objects.render(camera, i);
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
    model: dust::objects::ShadedMesh,
    model_wireframe: dust::objects::Wireframe,
    fire_model: dust::objects::ShadedMesh,
    fire_model_wireframe: dust::objects::Wireframe,
    result_model: dust::objects::ShadedMesh,
    result_model_wireframe: dust::objects::Wireframe,
}

impl Objects
{
    fn new(gl: &gl::Gl, model_meshes: String, fire_mesh: String, result_mesh: String) -> Objects
    {
        let (model, model_wireframe) = new_surface_and_wireframe(gl, model_meshes, &vec3(1.0, 0.0, 1.0));
        let (fire_model, fire_model_wireframe) = new_surface_and_wireframe(gl, fire_mesh, &vec3(0.0, 1.0, 1.0));
        let (result_model, result_model_wireframe) = new_surface_and_wireframe(gl, result_mesh, &vec3(0.0, 1.0, 0.0));

        Objects {model, model_wireframe, fire_model, fire_model_wireframe, result_model, result_model_wireframe}
    }

    fn render(&self, camera: &Camera, level: i32)
    {
        let model_matrix = Mat4::from_translation(vec3(0.0, 2.0, 0.0));
        match level {
            0 => {
                self.model.render(&model_matrix, camera);
                self.fire_model.render(&model_matrix, camera);
            },
            1 => {
                self.model_wireframe.render(camera);
                self.fire_model_wireframe.render(camera);
            },
            2 => {
                self.result_model_wireframe.render(camera);
            },
            3 => {
                self.result_model.render(&model_matrix, camera);
            },
            _ => {}
        }
    }
}

fn new_surface_and_wireframe(gl: &gl::Gl, source: String, color: &dust::Vec3) -> (ShadedMesh, Wireframe)
{
    let mut model = ShadedMesh::new_from_obj_source(gl, source.clone()).unwrap();
    model.color = *color;

    let mut wireframe = Wireframe::new_from_obj_source(gl, source, 0.02, &vec3(0.0, 2.0, 0.0));
    wireframe.set_parameters(0.8, 0.2, 5.0);
    wireframe.set_color(color);
    (model, wireframe)
}

fn stitch(path: &str, out_folder: &str) -> (Mesh, Mesh)
{
    let model_file_name = "model.obj";
    let fire_file_name = "fire.json";
    let out_model_file_name = "result.obj";
    let out_fire_model_name = "fire.obj";

    println!("Started preprocessing of case in {}", path);
    println!("Output is found at: {}", out_folder);

    // Load model
    let source = std::fs::read_to_string(&format!("{}{}", path, model_file_name)).unwrap_or_else(
    |err| {
            eprintln!("Cannot load {} in {}: {:#?}", model_file_name, path, err);
            std::process::exit(2);
        }
    );

    let mut in_mesh = MeshBuilder::new().with_obj(source).build().unwrap();
    in_mesh.merge_overlapping_primitives().unwrap();
    //in_mesh.collapse_small_faces(0.01);
    //in_mesh.remove_lonely_primitives();
    //::in_mesh::test_utility::test_is_valid(&in_mesh).unwrap();

    // Fix in_mesh
    //in_mesh.smooth_vertices(0.1);
    //in_mesh.flip_edges(0.9);
    //in_mesh.collapse_small_faces(0.01);
    //in_mesh.remove_lonely_primitives();
    //::in_mesh::test_utility::test_is_valid(&in_mesh).unwrap();

    exporter::save(&in_mesh, &format!("{}in_mesh.obj", out_folder)).unwrap();

    // Load fire
    let fire = load_fire(&format!("{}{}", path, fire_file_name)).unwrap_or_else(
    |err| {
            eprintln!("Cannot load {} in {}: {:#?}", fire_file_name, path, err);
            std::process::exit(2);
        }
    );
    println!("Fire: {:?}", fire);

    // Create fire model
    let mut fire_mesh = tri_mesh::MeshBuilder::new().icosahedron().build().unwrap();
    fire_mesh.scale(fire.radius);
    fire_mesh.translate(fire.position);

    exporter::save(&fire_mesh, &format!("{}{}", out_folder, out_fire_model_name)).unwrap_or_else(
    |err| {
            eprintln!("Cannot save {} in {}: {:#?}", out_fire_model_name, out_folder, err);
            std::process::exit(2);
        }
    );

    // Stitching
    println!("");
    println!("Stitching in_mesh: Vertices: {:?} and Faces: {:?}", in_mesh.no_vertices(), in_mesh.no_faces());
    println!("with fire_mesh: Vertices: {:?} and Faces: {:?}", fire_mesh.no_vertices(), fire_mesh.no_faces());

    let mut out_mesh = fire_mesh.clone();
    let (mut out_part_meshes, mut in_part_meshes) = out_mesh.split_at_intersection(&mut in_mesh);

    print_and_save(&out_part_meshes, &format!("out_mesh"), out_folder);
    print_and_save(&in_part_meshes, &format!("in_mesh"), out_folder);

    let mut meshes_to_merge = Vec::new();

    for submesh in out_part_meshes.drain(..) {
        if mesh_is_inside_other(&submesh, &in_mesh, &fire.position)
        {
            meshes_to_merge.push(submesh);
        }
    }

    for submesh in in_part_meshes.drain(..) {
        if mesh_is_inside_other(&submesh, &out_mesh, &fire.position)
        {
            meshes_to_merge.push(submesh);
        }
    }

    println!("Meshes to merge: {}", meshes_to_merge.len());
    print_and_save(&meshes_to_merge, &format!("mesh_to_merge"), out_folder);

    let mut iter = meshes_to_merge.drain(..);
    let mut result_mesh = iter.next().unwrap();
    for mesh in iter {
        result_mesh.merge_with(&mesh).unwrap_or_else(
            |err| {
                    println!("Error in merging: {:?}", err);
                    panic!("Error in merging: {:?}", err);
                }
            );
    }

    println!("Result in_mesh: Vertices: {:?} and Faces: {:?}", result_mesh.no_vertices(), result_mesh.no_faces());

    // Save in_mesh
    exporter::save(&result_mesh, &format!("{}{}", out_folder, out_model_file_name)).unwrap_or_else(
    |err| {
            eprintln!("Cannot save {} in {}: {:#?}", out_model_file_name, out_folder, err);
            std::process::exit(2);
        }
    );
    (fire_mesh, result_mesh)
}

fn print_and_save(meshes: &Vec<tri_mesh::mesh::Mesh>, name: &str, folder: &str)
{
    let mut i = 0;
    for mesh in meshes {
        println!("Mesh {} {}: Vertices: {:?} and Faces: {:?}", name, i, mesh.no_vertices(), mesh.no_faces());
        exporter::save(mesh, &format!("{}{}{}.obj", folder, name, i)).unwrap();
        i += 1;
    }
}

fn mesh_is_inside_other(mesh: &tri_mesh::mesh::Mesh, other: &tri_mesh::mesh::Mesh, point: &Vec3) -> bool
{
    for face_id in mesh.face_iter() {
        let face_center = mesh.face_center(face_id);
        if !mesh_blocks_view(other, &face_center, point)
        {
            return true;
        }
    }
    false
}

fn mesh_blocks_view(mesh: &tri_mesh::mesh::Mesh, point0: &Vec3, point1: &Vec3) -> bool
{
    for face_id in mesh.face_iter() {
        if mesh.face_line_piece_intersection(face_id, point0, point1).is_some()
        {
            return true;
        }
    }
    false
}

#[derive(Debug)]
struct Fire {
    position: Vec3,
    radius: f64
}

fn load_fire(filename: &str) -> Result<Fire, serde_json::Error>
{
    let data = load_json(filename);
    let obj: serde_json::Value = serde_json::from_str(&data)?;
    let p = &obj["position"];
    let position = vec3(p["x"].as_f64().unwrap(), p["y"].as_f64().unwrap(), p["z"].as_f64().unwrap());
    let radius = obj["radius"].as_f64().unwrap();
    let n = &obj["normal"];
    let normal = vec3(n["x"].as_f64().unwrap(), n["y"].as_f64().unwrap(), n["z"].as_f64().unwrap());
    Ok(Fire { position: position + normal * 0.5 * radius, radius })
}

fn load_json(filename: &str) -> String
{
    use std::io::prelude::*;
    let mut f = std::fs::File::open(filename).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    contents
}