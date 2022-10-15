use bevy::{prelude::*};
use crate::GameState;
use crate::camera::{SlidesCamera};
use crate::player::{Player};
use crate::backgrounds::{Background};

#[derive(Component, Deref, DerefMut)]
pub(crate) struct SlideTimer {
   pub timer: Timer,
}

#[derive(Component)]
pub(crate) struct SlideDeck {
    pub total_slides: usize,
    pub current_slide: usize,
}

#[derive(Component)]
pub(crate) struct Slide;

pub(crate) struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
		app
        .add_system_set(SystemSet::on_enter(GameState::Credits)
            .with_system(setup_credits))
        .add_system_set(SystemSet::on_update(GameState::Credits)
            .with_system(show_slide)
            .with_system(handle_exit_slides))
		.add_system_set(SystemSet::on_exit(GameState::Credits)
			.with_system(despawn_credits));
	}
}

pub(crate) fn setup_credits(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<SlidesCamera>, Without<Player>, Without<Background>)>
) {
    // Despawn all non-slides cameras
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(SlidesCamera);

    let slides = vec![
        "credits/gavin_credit.png",
        "credits/dan_credit.png",
        "credits/camryn_credit.png",
        "credits/caela_credit.png",
        "credits/prateek_credit.png",
        "credits/chase_credit.png",
        "credits/nathan_credit.png",
        "credits/chris_credit.png",
    ];

    for i in 0..slides.len() {
        commands.spawn_bundle(SpriteBundle {
            texture: asset_server.load(slides[i]),
            visibility: Visibility {
                is_visible: if i == 0 { true } else { false },
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        }).insert(Slide);
    }

    commands.spawn().insert(SlideTimer {
        timer: Timer::from_seconds(5.0, true),
    });
    commands.spawn().insert(SlideDeck {
        total_slides: slides.len(),
        current_slide: 1,
    });
}

pub(crate) fn despawn_credits(mut commands: Commands,
	camera_query: Query<Entity,  With<SlidesCamera>>,
    timer_query: Query<Entity, With<SlideTimer>>,
    deck_query: Query<Entity, With<SlideDeck>>,
    slides_query: Query<Entity, With<Slide>>
) {
    // Despawn credits camera
    camera_query.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Despawn timers
    timer_query.for_each(|timer| {
        commands.entity(timer).despawn();
    });

    // Despawn slidedeck
    deck_query.for_each(|deck| {
        commands.entity(deck).despawn();
    });

    // Despawn slide sprites
    slides_query.for_each(|slide| {
        commands.entity(slide).despawn();
    });
}

pub(crate) fn show_slide(
    time: Res<Time>,
    mut slide_timer: Query<&mut SlideTimer>,
    mut visibility: Query<&mut Visibility>,
    mut slide_deck: Query<&mut SlideDeck>,
) {
    // .single() is forceful: if the queries are empty, the unwrap panics.
    if slide_timer.is_empty() || visibility.is_empty() || slide_deck.is_empty() {
        return;
    }

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
                if slide.current_slide < max_slide_number - 1 {
                    slide.current_slide += 1;
                } else {
                    slide.current_slide = 0;
                }
            }
        }
    }
}

fn handle_exit_slides(
	input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>
) {
    if input.pressed(KeyCode::Escape) {
        // Hope this doesn't fail :)
        state.set(GameState::Start);
    }
}
