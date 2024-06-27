use gamestate::GameState;
use ruscii::app::{App, State};
use ruscii::drawing::Pencil;
use ruscii::gui::FPSCounter;
use ruscii::keyboard::{Key, KeyEvent};
use ruscii::terminal::Window;

mod cell;
mod gamestate;

fn main() {
    let mut fps_counter = FPSCounter::default();
    let mut app = App::default();
    // let mut app = App::config(Config { fps: 30 });
    let win_size = app.window().size();
    let mut state = GameState::new(win_size);
    state.init();

    app.run(|app_state: &mut State, window: &mut Window| {
        for key_event in app_state.keyboard().last_key_events() {
            match key_event {
                KeyEvent::Pressed(Key::Esc) => app_state.stop(),
                KeyEvent::Pressed(Key::Q) => app_state.stop(),
                _ => (),
            }
        }

        fps_counter.update();
        let step = app_state.step();
        let mut pencil = Pencil::new(window.canvas_mut());

        state.set_step(step);
        state.handle_keys_down(app_state.keyboard().get_keys_down());
        state.update();
        state.draw(&mut pencil);
    });
}
