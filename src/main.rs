use specs::prelude::*;
use tetra::graphics::{self, Color, Texture, DrawParams, Rectangle};
use tetra::{Context, ContextBuilder, State};
use tetra::time::Timestep;
use tetra::math::Vec2;

const SPRITE_SIZE: i32 = 20;
const SCREEN_SIZE: i32 = 20;
const INITIAL_TAIL: usize = 5;

// A component contains data
// which is associated with an entity.
#[derive(Debug)]
struct Velocity(Vec2<i32>);

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
struct Lifetime(usize);

impl Component for Lifetime {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
struct Position(Vec2<i32>);

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Sprite {
    rect: Rectangle,
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}

struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    // These are the resources required for execution.
    // You can also define a struct and `#[derive(SystemData)]`,
    // see the `full` example.
    type SystemData = (WriteStorage<'a, Position>, ReadStorage<'a, Velocity>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        // The `.join()` combines multiple component storages,
        // so we get access to all entities which have
        // both a position and a velocity.
        for (pos, vel) in (&mut pos, &vel).join() {
            (pos.0).x = ((pos.0).x + (vel.0).x + SCREEN_SIZE) % SCREEN_SIZE;
            (pos.0).y = ((pos.0).y + (vel.0).y + SCREEN_SIZE) % SCREEN_SIZE;
        }
    }
}

struct LifetimeSystem;

impl<'a> System<'a> for LifetimeSystem {
    type SystemData = (
        WriteStorage<'a, Lifetime>,
        Entities<'a>
    );

    fn run(&mut self, (mut lifetimes, entities): Self::SystemData) {
        for (lifetime, entity) in (&mut lifetimes, &*entities).join() {
            if lifetime.0 > 0 {
                lifetime.0 -= 1;
            } else {
                entities.delete(entity);
            }
        }
    }
}

struct GameState<'a> {
    world: World,
    dispatcher: Dispatcher<'a, 'a>,
    spritesheet: Texture,
}

impl<'a> GameState<'a> {
    fn new(ctx: &mut Context) -> tetra::Result<Self> {
        // The `World` is our
        // container for components
        // and other resources.
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Velocity>();
        world.register::<Lifetime>();
        world.register::<Sprite>();

        // An entity may or may not contain some component.
        world.create_entity()
            .with(Velocity(Vec2::new(1, 0)))
            .with(Position(Vec2::new(0, 0)))
            .with(Lifetime(INITIAL_TAIL))
            .with(Sprite{rect: Rectangle::new(0.0,0.0,1.0,1.0)})
            .build();
        // world.create_entity().with(Vel(Vec2::new(0.0, 1.0))).with(Pos(Vec2::new(3.0, 2.0))).build();
        // world.create_entity().with(Vel(Vec2::new(-1.0, 2.0))).with(Pos(Vec2::new(5.0, 4.0))).build();

        // This entity does not have `Vel`, so it won't be dispatched.
        world.create_entity()
            .with(Position(Vec2::new(2, 0)))
            .with(Sprite{rect: Rectangle::new(0.0,1.0,1.0,1.0)})
            .build();

        // This builds a dispatcher.
        // The third parameter of `with` specifies
        // logical dependencies on other systems.
        // Since we only have one, we don't depend on anything.
        // See the `full` example for dependencies.
        let mut dispatcher = DispatcherBuilder::new()
            .with(MovementSystem, "movement", &[])
            .with(LifetimeSystem, "lifetime", &[])
            //.with(RenderSystem::new(), "renderer", &[])
            .build();
        // This will call the `setup` function of every system.
        // In this example this has no effect since we already registered our components.
        dispatcher.setup(&mut world);
        let spritesheet = Texture::new(ctx, "./assets/spritesheet.png")?;

        Ok(
            Self {
                world,
                dispatcher,
                spritesheet
            }
        )
    }
}

impl<'a> State for GameState<'a> {
    fn update(&mut self, _ctx: &mut Context) -> tetra::Result {
        // This dispatches all the systems in parallel (but blocking).
        self.dispatcher.dispatch(&mut self.world);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);

        let positions = self.world.read_storage::<Position>();
        let sprites = self.world.read_storage::<Sprite>();
        
        let scale = Vec2::new((SPRITE_SIZE - 1) as f32 , (SPRITE_SIZE - 1) as f32);

        for (position, sprite) in (&positions, &sprites).join() {
            let pos = Vec2::new(((position.0).x * SPRITE_SIZE) as f32, ((position.0).y * SPRITE_SIZE) as f32);
                graphics::draw(ctx, &self.spritesheet, DrawParams::new()
                    .position(pos)
                    .clip(sprite.rect)
                    .scale(scale));
        }


        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Specs + Tetra", SPRITE_SIZE * SCREEN_SIZE, SPRITE_SIZE * SCREEN_SIZE)
        .timestep(Timestep::Fixed(15.0))
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}