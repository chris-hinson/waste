#[warn(unused_imports)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::{
	GameState
};
use crate::camera::{MenuCamera};
use crate::player::{Player};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Background
};

//for now, text for buttons is black, but that can be changed here:
const TEXT_COLOR: Color = Color::rgb(0.,0.,0.);
const MENU_BACKGROUND: &str = "backgrounds/start_screen.png";

pub struct MainMenuPlugin;
//sets up Handlers for fonts and images for a button.
struct UiAssets{
	font: Handle<Font>,
	button: Handle<Image>,
	button_pressed: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct MainMenuBackground;

#[derive(Component)]
pub(crate) struct StartButton;

#[derive(Component)]
pub(crate) struct CreditsButton;

//Builds plugin called MainMenuPlugin
impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system(GameState::Start, setup_menu)
		.add_system_set(ConditionSet::new()
			// Only run handlers on Start state
			// TODO: This could be refactored to be cleaner following the
			// example in iyes_loopless:
			// https://github.com/IyesGames/iyes_loopless/blob/main/examples/menu.rs
			.run_in_state(GameState::Start)
				.with_system(start_button_handler)
				.with_system(credits_button_handler)
			.into())
		.add_exit_system(GameState::Start, despawn_start_menu);
	}
}

// Clears buttons from screen when ran
// Should be run after START button is pressed
fn despawn_start_menu(mut commands: Commands,
	button_query: Query<Entity, With<Button>>,
	camera_query: Query<Entity,  With<MenuCamera>>,
	background_query: Query<Entity, With<MainMenuBackground>>
){
	// Despawn buttons
	for b in button_query.iter() {
		commands.entity(b).despawn_recursive();
	}
	// Despawn cameras
	for c in camera_query.iter() {
		commands.entity(c).despawn();
	}
	// Despawn Main Menu Background
	for bckg in background_query.iter() {
		commands.entity(bckg).despawn();
	}
}


fn start_button_handler(
	interaction_query: Query<(&Children, &Interaction, With<StartButton>), Changed<Interaction>>,
	mut image_query: Query<&mut UiImage>, 
	ui_assets: Res<UiAssets>,
){
	for(children, interaction, _) in interaction_query.iter() {
		//grabs children of button
		let child = children.iter().next().unwrap();
		//gets image of buttons
		let mut image = image_query.get_mut(*child).unwrap();

		//What happens when a button is interacted with
		match interaction {
			Interaction::Clicked =>{
				image.0 = ui_assets.button_pressed.clone();
				commands.insert_resource(NextState(GameState::Playing));
			},
			Interaction::Hovered=> {
				image.0 = ui_assets.button_pressed.clone();
			}
			Interaction::None => {
				image.0 = ui_assets.button.clone();
			}
		}
	}
}

fn credits_button_handler(
	interaction_query: Query<(&Children, &Interaction, With<CreditsButton>), Changed<Interaction>>,
	mut image_query: Query<&mut UiImage>, 
	ui_assets: Res<UiAssets>,
) {
	for(children, interaction, _) in interaction_query.iter() {
		//grabs children of button
		let child = children.iter().next().unwrap();
		//gets image of buttons
		let mut image = image_query.get_mut(*child).unwrap();

		//What happens when a button is interacted with
		match interaction {
			Interaction::Clicked =>{
				image.0 = ui_assets.button_pressed.clone();
				// Change to the credits state
				commands.insert_resource(NextState(GameState::Credits));
			},
			Interaction::Hovered=> {
				image.0 = ui_assets.button_pressed.clone();
			}
			Interaction::None => {
				image.0 = ui_assets.button.clone();
			}
		}
	}
}


fn setup_menu(mut commands: Commands,
	assets: Res<AssetServer>,
	cameras: Query<Entity, (With<Camera2d>, Without<MenuCamera>, Without<Player>, Without<Background>)>
){ 
	cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

	//TODO:
	//Choose actual font and button images
	//gives font and images for start button:
	let ui_assets = UiAssets {
		font: assets.load("buttons/joystix monospace.ttf"),
		button: assets.load("buttons/start_button.png"),
		button_pressed: assets.load("buttons/start_button_pressed.png"),
	};

	//creates camera for UI
	let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MenuCamera);

	commands.spawn_bundle(SpriteBundle {
		texture: assets.load(MENU_BACKGROUND),
		transform: Transform::from_xyz(0., 0., 0.),
		..default()
	})
	.insert(MainMenuBackground);
	
	//START BUTTON:
	//Note that the button comes in two parts: the clickable part and the image part.
	//The image part will be a child of the clickable part.
	//CLICKABLE PART OF START BUTTON:
	commands.spawn_bundle(ButtonBundle {
		//sets up Style for button so that it's in the center of the screen.
		//documentation for Style can be found here: https://docs.rs/bevy/0.1.2/bevy/prelude/struct.Style.html
		style: Style {  
			//aligns self and children in center
			align_items: AlignItems::Center, 
			align_self: AlignSelf::Center, 
			justify_content: JustifyContent::Center,
			margin: UiRect::all(Val::Auto),
			size: Size::new(Val::Percent(40.), Val::Percent(20.)), 
			..Default::default()
		},
		//makes clickable part invisible so that you can see the image part.
		color: Color::NONE.into(),
		..Default::default()
	})
	//IMAGE PART OF START BUTTON:
	.with_children(|parent| {
		parent.spawn_bundle(ImageBundle
		{
			style: Style {
				size: Size::new(Val::Percent(100.), Val::Percent(100.)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..Default::default()
			},
			image: ui_assets.button.clone().into(),
			..Default::default()
		})
		//Prevents image of button from blocking the button for a mouse click.
		.insert(FocusPolicy::Pass)
		//Creates the TEXT on the button
		.with_children(|parent| 
		{
			parent.spawn_bundle( TextBundle 
			{
				text: Text::from_section
				(
					//Text on the START Button
					"Enter the Wastes",
					TextStyle {
						font: ui_assets.font.clone(),
						font_size: 40.,
						color: TEXT_COLOR,
						..Default::default()
					},
				),
				focus_policy: FocusPolicy::Pass,
				..Default::default()
			});
		});
	}).insert(StartButton);


	//CREDITS button:
	commands.spawn_bundle(ButtonBundle {
			style: Style {  
			align_items: AlignItems::Center, 
			align_self: AlignSelf::Center, 
			justify_content: JustifyContent::Center,
			margin: UiRect::all(Val::Auto),
			//button needs to be absolutely aligned
			position_type: PositionType::Absolute,
			position: UiRect {
				bottom: Val::Px(100.),
				left: Val::Px((WIN_W * 0.8) / 2.),
				..default()
			},
			//CREDITS button is smaller than START button:
			size: Size::new(Val::Percent(20.), Val::Percent(10.)), 
			..Default::default()
		},
		color: Color::NONE.into(),
		..Default::default()
	})
	//IMAGE PART of CREDITS button:
	.with_children(|parent| {
		parent.spawn_bundle(ImageBundle
		{
			style: Style {
				size: Size::new(Val::Percent(100.), Val::Percent(100.)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..Default::default()
			},
			image: ui_assets.button.clone().into(),
			..Default::default()
		})
		.insert(FocusPolicy::Pass)
		//TEXT part of CREDITS button
		.with_children(|parent| 
		{
			parent.spawn_bundle( TextBundle 
			{
				text: Text::from_section
				(
					//Text on the START Button
					"Credits",
					TextStyle {
						font: ui_assets.font.clone(),
						//font size of credits must be smaller than START
						font_size: 30.,
						color: TEXT_COLOR,
						..Default::default()
					},
				),
				focus_policy: FocusPolicy::Pass,
				..Default::default()
			});
		});
	}).insert(CreditsButton);
	//END CREDITS BUTTON

	//adds resources to the App
	commands.insert_resource(ui_assets);
}