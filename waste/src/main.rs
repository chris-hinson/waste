
use bevy::{
	prelude::*,
	window::PresentMode,
};

#[derive(Component, Deref, DerefMut)]
struct SlideTimer{
    timer: Timer,
}

#[derive(Component)]
struct SlideDeck{
    total_slides: usize,
    current_slide: usize,
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
        "dan_credit.png",
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
    commands.spawn().insert(SlideDeck{total_slides:slides.len(), current_slide: 0});
	
	
}


fn show_slide(
    time: Res<Time>,
    mut slide_timer: Query<&mut SlideTimer>,
    mut visibility: Query<&mut Visibility>,
    mut slide_deck: Query<&mut SlideDeck>,
){
    // Query gets all the components that match the type
    // i.e. Query<&mut Visibility> gets all visibility components(length of slide deck)
    // components without visibility are not queried(still needs to be verified)
    // if there is only one, we can use .single() / .single_mut()
    let max_slide_number = slide_deck.single().total_slides;
    for mut timer in slide_timer.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            for mut slide in slide_deck.iter_mut() {
                for (index, mut current_slide_visibility) in (visibility.iter_mut()).enumerate() {
                    // only the matching slide is visible
                    if index == slide.current_slide {
                        current_slide_visibility.is_visible = true;
                    } else {
                        current_slide_visibility.is_visible = false;
                    }
                }
                // loop back to the first slide
                if slide.current_slide < max_slide_number-1 {
                    slide.current_slide += 1;
                } else {
                    slide.current_slide = 0;
                }
            }
        }
    }

}