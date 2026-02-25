use soroban_sdk::{Address, BytesN, Env, String};

/// Emit when a multi-sig certificate request is created.
pub fn emit_multisig_request_created(
    env: &Env,
    request_id: &BytesN<32>,
    requester: &Address,
    course_id: &String,
) {
    let topics = (
        soroban_sdk::Symbol::new(env, "multisig_request_created"),
        requester.clone(),
    );
    env.events()
        .publish(topics, (request_id.clone(), course_id.clone()));
}

/// Emit when an approval is granted.
pub fn emit_multisig_approval_granted(
    env: &Env,
    request_id: &BytesN<32>,
    approver: &Address,
    current: u32,
    required: u32,
) {
    let topics = (
        soroban_sdk::Symbol::new(env, "multisig_approval_granted"),
        approver.clone(),
    );
    env.events()
        .publish(topics, (request_id.clone(), current, required));
}

/// Emit when a request is rejected by an approver.
pub fn emit_multisig_request_rejected(env: &Env, request_id: &BytesN<32>, approver: &Address) {
    let topics = (
        soroban_sdk::Symbol::new(env, "multisig_request_rejected"),
        approver.clone(),
    );
    env.events().publish(topics, request_id.clone());
}

/// Emit when a request reaches full approval.
pub fn emit_multisig_request_approved(env: &Env, request_id: &BytesN<32>) {
    let topics = (soroban_sdk::Symbol::new(env, "multisig_request_approved"),);
    env.events().publish(topics, request_id.clone());
}

/// Emit when a certificate is issued via multi-sig.
pub fn emit_certificate_issued(
    env: &Env,
    certificate_id: &BytesN<32>,
    student: &Address,
    course_id: &String,
) {
    let topics = (
        soroban_sdk::Symbol::new(env, "certificate_issued"),
        student.clone(),
    );
    env.events()
        .publish(topics, (certificate_id.clone(), course_id.clone()));
}

/// Emit when a certificate is revoked.
pub fn emit_certificate_revoked(env: &Env, certificate_id: &BytesN<32>, revoked_by: &Address) {
    let topics = (
        soroban_sdk::Symbol::new(env, "certificate_revoked"),
        revoked_by.clone(),
    );
    env.events().publish(topics, certificate_id.clone());
}

/// Emit when a certificate is reissued.
pub fn emit_certificate_reissued(
    env: &Env,
    old_id: &BytesN<32>,
    new_id: &BytesN<32>,
    student: &Address,
) {
    let topics = (
        soroban_sdk::Symbol::new(env, "certificate_reissued"),
        student.clone(),
    );
    env.events()
        .publish(topics, (old_id.clone(), new_id.clone()));
}

/// Emit when a multi-sig config is updated.
pub fn emit_multisig_config_updated(env: &Env, course_id: &String, admin: &Address) {
    let topics = (
        soroban_sdk::Symbol::new(env, "multisig_config_updated"),
        admin.clone(),
    );
    env.events().publish(topics, course_id.clone());
}

/// Emit when a batch operation completes.
pub fn emit_batch_completed(env: &Env, total: u32, succeeded: u32, failed: u32) {
    let topics = (soroban_sdk::Symbol::new(env, "batch_completed"),);
    env.events().publish(topics, (total, succeeded, failed));
}

/// Emit when a certificate is shared.
pub fn emit_certificate_shared(
    env: &Env,
    certificate_id: &BytesN<32>,
    shared_by: &Address,
    platform: &String,
) {
    let topics = (
        soroban_sdk::Symbol::new(env, "certificate_shared"),
        shared_by.clone(),
    );
    env.events()
        .publish(topics, (certificate_id.clone(), platform.clone()));
}

/// Emit when a compliance check is performed.
pub fn emit_compliance_checked(env: &Env, certificate_id: &BytesN<32>, is_compliant: bool) {
    let topics = (soroban_sdk::Symbol::new(env, "compliance_checked"),);
    env.events()
        .publish(topics, (certificate_id.clone(), is_compliant));
}

/// Emit when a certificate template is created.
pub fn emit_template_created(env: &Env, template_id: &String, created_by: &Address) {
    let topics = (
        soroban_sdk::Symbol::new(env, "template_created"),
        created_by.clone(),
    );
    env.events().publish(topics, template_id.clone());
}

/// Emit when a certificate is verified.
pub fn emit_certificate_verified(env: &Env, certificate_id: &BytesN<32>, is_authentic: bool) {
    let topics = (soroban_sdk::Symbol::new(env, "certificate_verified"),);
    env.events()
        .publish(topics, (certificate_id.clone(), is_authentic));
}
