use crossterm::{
    cursor::{Hide, Show},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rusty_audio::Audio;
use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
    {io, thread},
};

use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    invaders::Invaders,
    level::Level,
    menu::Menu,
    player::Player,
    render,
    score::Score,
};


fn render_screen(render_rx: Receiver<Frame>) {
    let mut last_frame = frame::new_frame();
    let mut stdout = io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);
    while let Ok(curr_frame) = render_rx.recv() {
        render::render(&mut stdout, &last_frame, &curr_frame, false);
        last_frame = curr_frame;
    }
}

fn reset_game(in_menu: &mut bool, player: &mut Player, invaders: &mut Invaders) {
    *in_menu = true;
    *player = Player::new();
    *invaders = Invaders::new();
}


fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    for item in &["explode", "lose", "move", "pew", "startup", "win"] {
        audio.add(item, &format!("audio/original/{}.ogg", item));
    }
    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        render_screen(render_rx);
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    let mut score = Score::new();
    let mut menu = Menu::new();
    let mut in_menu = true;
    let mut level = Level::new();

    'gameloop: loop {
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        
        if in_menu {


            while crossterm::event::poll(Duration::default())? {
                if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                    match key_event.code {
                        crossterm::event::KeyCode::Up => menu.change_option(true),
                        crossterm::event::KeyCode::Down => menu.change_option(false),
                        crossterm::event::KeyCode::Char(' ') | crossterm::event::KeyCode::Enter => {
                            if menu.selection == 0 {
                                in_menu = false;
                            } else {
                                break 'gameloop;
                            }
                        }
                        _ => {}
                    }
                }
            }
            menu.draw(&mut curr_frame);

            let _ = render_tx.send(curr_frame);
            thread::sleep(Duration::from_millis(1));
            continue;
        } else {
            while crossterm::event::poll(Duration::default())? {
                if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                    match key_event.code {
                        crossterm::event::KeyCode::Left => player.move_left(),
                        crossterm::event::KeyCode::Right => player.move_right(),
                        crossterm::event::KeyCode::Char(' ') | crossterm::event::KeyCode::Enter => {
                            if player.shoot() {
                                audio.play("pew");
                            }
                        }
                        crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                            audio.play("lose");
                            reset_game(&mut in_menu, &mut player, &mut invaders);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        let hits: u16 = player.detect_hits(&mut invaders);
        if hits > 0 {
            audio.play("explode");
            score.add_points(hits);
        }
        // Draw & render

        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders, &score, &level];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        // Win or lose?
        if invaders.all_killed() {
            if level.increment_level() {
                audio.play("win");
                break 'gameloop;
            }
            invaders = Invaders::new();
        } else if invaders.reached_bottom() {
            audio.play("lose");
            reset_game(&mut in_menu, &mut player, &mut invaders);
        }
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
