use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core_tests::helpers::with_user;

#[test]
fn test_user() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            fixture.vault.with_state(|state| {
                assert_eq!(state.user.user.as_ref().unwrap().id, fixture.user_id);
            });
        }
        .boxed()
    });
}
