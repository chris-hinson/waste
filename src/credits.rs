use bevy::{prelude::*};

#[derive(Component, Deref, DerefMut)]
pub struct SlideTimer {
   pub timer: Timer,
}

#[derive(Component)]
pub struct SlideDeck {
    pub total_slides: usize,
    pub current_slide: usize,
}

pub fn show_slide(
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
