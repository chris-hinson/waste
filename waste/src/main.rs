
use bevy::{
	prelude::*,
	window::PresentMode,
};

#[derive(Component, Deref, DerefMut)]
struct SlideTimer{
    timer: Timer,
}

#[derive(Component, Deref, DerefMut)]
struct Slide{
    slide: usize,
}

fn main() {
	App::new()
		.insert_resource(WindowDescriptor {
			title: String::from("Hello World!"),
			width: 1280.,
			height: 720.,
			present_mode: PresentMode::Fifo,
			..default()
		})
		.add_plugins(DefaultPlugins)
		.add_startup_system(setup)
		.add_system(show_slide)
		.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Printing credits");
	commands.spawn_bundle(Camera2dBundle::default());

    let slides = vec![
        "1.png",
        "2.png",
        "3.png",
        "4.png",
    ];

    for i in 0..slides.len() {
        commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(slides[i]),
            visibility: Visibility {
                is_visible: false,
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        });
    }

    commands.spawn().insert(SlideTimer{timer: Timer::from_seconds(2.0, true)});
    commands.spawn().insert(Slide{slide: 0});
	
	
}


fn show_slide(
    time: Res<Time>,
    mut slide_timer: Query<&mut SlideTimer>,
    mut visibility: Query<&mut Visibility>,
    mut slide: Query<&mut Slide>,
){
    // let slide_to_show: usize= 0;
    // for (mut timer, mut visibility) in change_slide.iter_mut() {
    //     timer.tick(time.delta());
    //     if timer.just_finished() {

    //     }
    // }
    let max_slide_number = 4;
    for mut timer in slide_timer.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            for mut slide in slide.iter_mut() {
                if slide.slide < max_slide_number {
                    slide.slide += 1;
                } else {
                    slide.slide = 0;
                }
                for (index, mut current_slide_visibility) in (visibility.iter_mut()).enumerate() {
                    if index == slide.slide {
                        current_slide_visibility.is_visible = true;
                    } else {
                        current_slide_visibility.is_visible = false;
                    }
                }
            }
        }
    }

}