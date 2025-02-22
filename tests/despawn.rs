use bevy::prelude::*;
use bevy_replicon::{
    client::server_entity_map::ServerEntityMap, prelude::*, test_app::ServerTestAppExt,
};
use serde::{Deserialize, Serialize};

#[test]
fn single() {
    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            RepliconPlugins.set(ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                ..Default::default()
            }),
        ));
    }

    server_app.connect_client(&mut client_app);

    let server_entity = server_app.world_mut().spawn(Replicated).id();
    let client_entity = client_app.world_mut().spawn(Replicated).id();

    client_app
        .world_mut()
        .resource_mut::<ServerEntityMap>()
        .insert(server_entity, client_entity);

    server_app.world_mut().despawn(server_entity);

    server_app.update();
    server_app.exchange_with_client(&mut client_app);
    client_app.update();

    assert!(client_app.world().entities().is_empty());

    let entity_map = client_app.world().resource::<ServerEntityMap>();
    assert!(entity_map.to_client().is_empty());
    assert!(entity_map.to_server().is_empty());
}

#[test]
fn with_heirarchy() {
    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            RepliconPlugins.set(ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                ..Default::default()
            }),
        ));
    }

    server_app.connect_client(&mut client_app);

    let server_child_entity = server_app.world_mut().spawn(Replicated).id();
    let server_entity = server_app
        .world_mut()
        .spawn(Replicated)
        .push_children(&[server_child_entity])
        .id();

    let client_child_entity = client_app.world_mut().spawn(Replicated).id();
    let client_entity = client_app
        .world_mut()
        .spawn(Replicated)
        .push_children(&[client_child_entity])
        .id();

    let mut entity_map = client_app.world_mut().resource_mut::<ServerEntityMap>();
    entity_map.insert(server_entity, client_entity);
    entity_map.insert(server_child_entity, client_child_entity);

    server_app.world_mut().despawn(server_entity);
    server_app.world_mut().despawn(server_child_entity);

    server_app.update();
    server_app.exchange_with_client(&mut client_app);
    client_app.update();

    server_app.world_mut().despawn(server_entity);
    server_app.world_mut().despawn(server_child_entity);

    assert!(client_app.world().entities().is_empty());
}

#[test]
fn after_spawn() {
    let mut server_app = App::new();
    let mut client_app = App::new();
    for app in [&mut server_app, &mut client_app] {
        app.add_plugins((
            MinimalPlugins,
            RepliconPlugins.set(ServerPlugin {
                tick_policy: TickPolicy::EveryFrame,
                ..Default::default()
            }),
        ))
        .replicate::<DummyComponent>();
    }

    server_app.connect_client(&mut client_app);

    // Insert and remove `Replicated` to trigger spawn and despawn for client at the same time.
    server_app
        .world_mut()
        .spawn((Replicated, DummyComponent))
        .remove::<Replicated>();

    server_app.update();
    server_app.exchange_with_client(&mut client_app);
    client_app.update();

    assert!(client_app.world().entities().is_empty());
}

#[derive(Component, Deserialize, Serialize)]
struct DummyComponent;
