use vault_core_tests::helpers::with_vault;

#[test]
fn test_user() {
    with_vault(|user_fixture| async move {
        user_fixture.load().await;

        user_fixture.vault.with_state(|state| {
            assert_eq!(state.user.user.as_ref().unwrap().id, user_fixture.user_id);
        });
    });
}
