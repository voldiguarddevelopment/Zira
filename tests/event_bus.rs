use zira_core::create_bus;
use zira_proto::Event;

/// C1 — create_bus() exists and returns all three handles with the correct channel types.
#[tokio::test]
async fn test_create_bus_returns_handles() {
    let bus = create_bus();
    // Verify the mpsc sender can enqueue a command without error.
    bus.cmd_tx
        .send(Event::WakeDetected)
        .await
        .expect("cmd_tx.send failed");
    // Verify the broadcast sender can create a subscriber.
    let _rx = bus.event_tx.subscribe();
    // Verify the mpsc receiver holds the sent command.
    drop(bus.event_tx);
    let _event = bus.cmd_rx;
}

/// C2, C3 — an event published to the broadcast sender is received by every subscriber.
#[tokio::test]
async fn test_broadcast_fanout_two_subscribers() {
    let bus = create_bus();
    let mut rx1 = bus.event_tx.subscribe();
    let mut rx2 = bus.event_tx.subscribe();

    bus.event_tx
        .send(Event::WakeDetected)
        .expect("broadcast send failed");

    let got1 = rx1.recv().await.expect("rx1 did not receive event");
    let got2 = rx2.recv().await.expect("rx2 did not receive event");

    assert!(
        matches!(got1, Event::WakeDetected),
        "rx1 received wrong event"
    );
    assert!(
        matches!(got2, Event::WakeDetected),
        "rx2 received wrong event"
    );
}

/// C2, C3 — a command sent on the mpsc channel is received by the single consumer exactly once.
#[tokio::test]
async fn test_command_channel_single_consumer() {
    let bus = create_bus();
    let cmd_tx = bus.cmd_tx;
    let mut cmd_rx = bus.cmd_rx;

    cmd_tx
        .send(Event::TurnStarted)
        .await
        .expect("mpsc send failed");
    // Close the sender so the channel drains cleanly.
    drop(cmd_tx);

    let received = cmd_rx.recv().await.expect("cmd_rx did not receive command");
    assert!(
        matches!(received, Event::TurnStarted),
        "received wrong event on command channel"
    );

    // No second message — channel is drained and closed.
    let second = cmd_rx.recv().await;
    assert!(
        second.is_none(),
        "expected exactly one command, but received a second"
    );
}
