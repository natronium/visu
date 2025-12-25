use bevy::{prelude::*, render::render_resource::TextureFormat};
use v4l::{Device, Format, FourCC, io::traits::OutputStream, prelude::MmapStream, video::Output};

pub struct VcamPlugin;

impl Plugin for VcamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VcamStream>()
            .init_resource::<CamBuffer>()
            .add_systems(Update, update_vcam_buffer);
    }
}

#[derive(Resource)]
pub struct CamBuffer(pub(crate) Handle<Image>);

impl FromWorld for CamBuffer {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();

        let mut image = Image::new_target_texture(1280, 720, TextureFormat::Rgba8UnormSrgb);
        image.clear(&[0x00,0xff,0xff,0x00]);
        let vcam_image = images.add(image);

        CamBuffer(vcam_image)
    }
}

#[derive(Resource)]
struct VcamStream(MmapStream<'static>, Format);

impl Default for VcamStream {
    fn default() -> Self {
        let dev = Device::new(0).expect("Failed to open device");
        let format = dev
            .set_format(&Format::new(1280, 720, FourCC::new(b"RGB4")))
            .unwrap();
        println!("{format}");

        let stream = MmapStream::new(&dev, v4l::buffer::Type::VideoOutput)
            .expect("could not construct buffer stream");

        Self(stream, format)
    }
}

fn update_vcam_buffer(
    mut vcam_stream: ResMut<VcamStream>,
    image: Res<CamBuffer>,
    images: Res<Assets<Image>>,
) {
    let _fmt = vcam_stream.1.clone();
    let stream = &mut vcam_stream.0;

    let Some(image) = images.get(&image.0) else {
        println!("Could not get camera buffer!! Skipping buffer shuffling");
        return;
    };

    let Ok((buf, _meta)) = OutputStream::next(stream) else {
        println!("Could not get v4l output stream! Skipping buffer shuffling");
        return;
    };

    let data = image.data.as_ref().unwrap();
    buf.copy_from_slice(data);
    // buf.fill(128);
}
