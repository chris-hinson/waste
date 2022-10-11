use bevy::ui::FocusPolicy;
#[warn(unused_imports)]
use bevy::{prelude::*,
			sprite::*,
			text::*};
use crate::{WIN_H, WIN_W};
//for now, text for buttons is black, but that can be changed here:
const TEXT_COLOR: Color = Color::rgb(0.,0.,0.);

pub struct MainMenuPlugin;
//sets up Handlers for fonts and images for a button.
struct UiAssets{
	font: Handle<Font>,
	button: Handle<Image>,
	button_pressed: Handle<Image>,
}

//Builds plugin called MainMenuPlugin
impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(setup_menu);
	}
}

fn setup_menu(mut commands: Commands, assets: Res<AssetServer>){ 
	//TODO:
	//Choose actual font and button images
	//gives font and images for start button:
	let ui_assets = UiAssets {
		font: assets.load("buttons/joystix monospace.ttf"),
		button: assets.load("buttons/start_button.png"),
		button_pressed: assets.load("buttons/start_button_pressed.png"),
	};

	//creates camera for UI
	commands.spawn_bundle(Camera2dBundle::default());
	
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
	});


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
	});
	//END CREDITS BUTTON

	//adds resources to the App
	commands.insert_resource(ui_assets);
}