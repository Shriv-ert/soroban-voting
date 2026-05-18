#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ============================================================
//  Helper: Setup contract + admin
// ============================================================
fn setup_contract(env: &Env) -> (SecureVotingContractClient, Address) {
    let contract_id = env.register(SecureVotingContract, ());
    let client = SecureVotingContractClient::new(env, &contract_id);
    let admin = Address::generate(env);

    // Inisialisasi contract dengan admin
    client.initialize(&admin);

    (client, admin)
}

// ============================================================
//  Test 1: Inisialisasi contract
// ============================================================
#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    // Verifikasi admin tersimpan
    assert_eq!(client.get_admin(), admin);

    // Verifikasi status awal = NotStarted
    assert_eq!(client.get_status(), VotingStatus::NotStarted);

    // Verifikasi total voters = 0
    assert_eq!(client.get_total_voters(), 0);
}

// ============================================================
//  Test 2: Tidak bisa inisialisasi 2 kali
// ============================================================
#[test]
#[should_panic(expected = "Contract sudah diinisialisasi!")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_contract(&env);

    // Coba inisialisasi lagi — harus PANIC
    let fake_admin = Address::generate(&env);
    client.initialize(&fake_admin);
}

// ============================================================
//  Test 3: Admin bisa tambah kandidat
// ============================================================
#[test]
fn test_add_candidate() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    // Tambah 2 kandidat
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));

    let candidates = client.get_candidates();
    assert_eq!(candidates.len(), 2);
    assert_eq!(
        candidates.get(0).unwrap().name,
        String::from_str(&env, "Kandidat A")
    );
    assert_eq!(candidates.get(0).unwrap().votes, 0);
}

// ============================================================
//  Test 4: Non-admin TIDAK bisa tambah kandidat
// ============================================================
#[test]
#[should_panic(expected = "Hanya admin yang bisa melakukan ini!")]
fn test_non_admin_cannot_add_candidate() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_contract(&env);

    // Orang random coba tambah kandidat — harus GAGAL
    let random_user = Address::generate(&env);
    client.add_candidate(&random_user, &String::from_str(&env, "Kandidat Palsu"));
}

// ============================================================
//  Test 5: Tidak bisa tambah kandidat setelah voting dimulai
// ============================================================
#[test]
#[should_panic(expected = "Tidak bisa tambah kandidat setelah voting dimulai!")]
fn test_cannot_add_candidate_after_open() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));
    client.open_voting(&admin);

    // Coba tambah kandidat setelah voting dibuka — harus GAGAL
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat C"));
}

// ============================================================
//  Test 6: Buka voting (perlu minimal 2 kandidat)
// ============================================================
#[test]
fn test_open_voting() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));

    client.open_voting(&admin);
    assert_eq!(client.get_status(), VotingStatus::Open);
}

// ============================================================
//  Test 7: Tidak bisa buka voting dengan kurang dari 2 kandidat
// ============================================================
#[test]
#[should_panic(expected = "Minimal harus ada 2 kandidat")]
fn test_cannot_open_with_less_than_2_candidates() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));

    // Hanya 1 kandidat — harus GAGAL
    client.open_voting(&admin);
}

// ============================================================
//  Test 8: Vote berhasil
// ============================================================
#[test]
fn test_vote_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));
    client.open_voting(&admin);

    // Ambil ID kandidat A
    let candidates = client.get_candidates();
    let candidate_a_id = candidates.get(0).unwrap().id;

    // Voter 1 vote kandidat A
    let voter1 = Address::generate(&env);
    client.vote(&voter1, &candidate_a_id);

    // Verifikasi
    assert_eq!(client.has_voted(&voter1), true);
    assert_eq!(client.get_total_voters(), 1);

    let updated = client.get_candidates();
    assert_eq!(updated.get(0).unwrap().votes, 1); // Kandidat A = 1 vote
    assert_eq!(updated.get(1).unwrap().votes, 0); // Kandidat B = 0 vote
}

// ============================================================
//  Test 9: TIDAK bisa vote 2 kali (anti double-voting)
// ============================================================
#[test]
#[should_panic(expected = "Kamu sudah vote!")]
fn test_cannot_vote_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));
    client.open_voting(&admin);

    let candidates = client.get_candidates();
    let id_a = candidates.get(0).unwrap().id;
    let id_b = candidates.get(1).unwrap().id;

    let voter = Address::generate(&env);
    client.vote(&voter, &id_a); // Vote pertama — OK
    client.vote(&voter, &id_b); // Vote kedua — harus GAGAL!
}

// ============================================================
//  Test 10: TIDAK bisa vote sebelum voting dibuka
// ============================================================
#[test]
#[should_panic(expected = "Voting belum dibuka atau sudah ditutup!")]
fn test_cannot_vote_before_open() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));

    // Belum open_voting! Langsung vote — harus GAGAL
    let voter = Address::generate(&env);
    client.vote(&voter, &0);
}

// ============================================================
//  Test 11: TIDAK bisa vote setelah voting ditutup
// ============================================================
#[test]
#[should_panic(expected = "Voting belum dibuka atau sudah ditutup!")]
fn test_cannot_vote_after_close() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.add_candidate(&admin, &String::from_str(&env, "Kandidat A"));
    client.add_candidate(&admin, &String::from_str(&env, "Kandidat B"));
    client.open_voting(&admin);
    client.close_voting(&admin); // Tutup voting

    let candidates = client.get_candidates();
    let id_a = candidates.get(0).unwrap().id;

    // Voting sudah ditutup — harus GAGAL
    let voter = Address::generate(&env);
    client.vote(&voter, &id_a);
}

// ============================================================
//  Test 12: Full flow — beberapa voter, lalu tutup, cek hasil
// ============================================================
#[test]
fn test_full_voting_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    // === FASE 1: Persiapan ===
    client.add_candidate(&admin, &String::from_str(&env, "Budi"));
    client.add_candidate(&admin, &String::from_str(&env, "Ani"));
    client.add_candidate(&admin, &String::from_str(&env, "Citra"));

    assert_eq!(client.get_candidates().len(), 3);
    assert_eq!(client.get_status(), VotingStatus::NotStarted);

    // === FASE 2: Buka Voting ===
    client.open_voting(&admin);
    assert_eq!(client.get_status(), VotingStatus::Open);

    // Ambil ID kandidat
    let candidates = client.get_candidates();
    let id_budi = candidates.get(0).unwrap().id;
    let id_ani = candidates.get(1).unwrap().id;
    let id_citra = candidates.get(2).unwrap().id;

    // === FASE 3: Voting ===
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);
    let voter4 = Address::generate(&env);
    let voter5 = Address::generate(&env);

    client.vote(&voter1, &id_ani);   // Voter 1 → Ani
    client.vote(&voter2, &id_budi);  // Voter 2 → Budi
    client.vote(&voter3, &id_ani);   // Voter 3 → Ani
    client.vote(&voter4, &id_citra); // Voter 4 → Citra
    client.vote(&voter5, &id_ani);   // Voter 5 → Ani

    // === FASE 4: Tutup Voting ===
    client.close_voting(&admin);
    assert_eq!(client.get_status(), VotingStatus::Closed);

    // === FASE 5: Verifikasi Hasil ===
    let results = client.get_candidates();
    assert_eq!(client.get_total_voters(), 5);

    // Budi = 1, Ani = 3, Citra = 1
    assert_eq!(results.get(0).unwrap().votes, 1);  // Budi
    assert_eq!(results.get(1).unwrap().votes, 3);  // Ani ← PEMENANG
    assert_eq!(results.get(2).unwrap().votes, 1);  // Citra

    // Verifikasi semua voter tercatat sudah vote
    assert_eq!(client.has_voted(&voter1), true);
    assert_eq!(client.has_voted(&voter2), true);
    assert_eq!(client.has_voted(&voter3), true);
    assert_eq!(client.has_voted(&voter4), true);
    assert_eq!(client.has_voted(&voter5), true);

    // Voter baru yang belum vote
    let voter_baru = Address::generate(&env);
    assert_eq!(client.has_voted(&voter_baru), false);
}
