use bevy::{
    prelude::*,
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{TextureFormat, TextureUsages},
    },
};
use v4l::{Device, Format, FourCC, io::traits::OutputStream, prelude::MmapStream, video::Output};

pub struct VcamPlugin;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const BYTES_PER_PIXEL: u32 = 4; //RGBA

impl Plugin for VcamPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CamBuffer>()
            .init_resource::<CamBuffer>()
            .add_systems(Startup, setup_readback);
    }
}

#[derive(Resource, Reflect)]
pub struct CamBuffer(pub(crate) Handle<Image>);

impl FromWorld for CamBuffer {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();

        let mut image = Image::new_target_texture(WIDTH, HEIGHT, TextureFormat::Bgra8UnormSrgb);
        image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
        image.clear(&[0x00, 0xff, 0xff, 0x00]);
        let vcam_image = images.add(image);

        CamBuffer(vcam_image)
    }
}

fn setup_readback(mut commands: Commands, vcam_image: Res<CamBuffer>) {
    let mut stream = init_v4l_stream();

    commands
        .spawn((
            Name::new("Vcam Texture Readback"),
            Readback::Texture(vcam_image.0.clone()),
        ))
        .observe(move |event: On<ReadbackComplete>| {
            if let Ok((buf, _meta)) = OutputStream::next(&mut stream) {
                buf.copy_from_slice(event.as_slice());
            };
        });
}

fn init_v4l_stream() -> MmapStream<'static> {
    let dev = Device::new(0).expect("Failed to open device");
    let format = dev
        .set_format(&Format::new(WIDTH, HEIGHT, FourCC::new(b"AR24")))
        .unwrap();
    println!("{format}");

    let stream = MmapStream::new(&dev, v4l::buffer::Type::VideoOutput)
        .expect("could not construct buffer stream");

    stream
}
