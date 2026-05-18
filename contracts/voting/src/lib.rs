#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Vec,
};

// ============================================================
//  TIPE DATA
// ============================================================

/// Status voting: Belum Dimulai, Sedang Berlangsung, atau Selesai
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum VotingStatus {
    NotStarted, // Belum dimulai — admin bisa tambah/hapus kandidat
    Open,       // Sedang berlangsung — user bisa vote
    Closed,     // Selesai — tidak bisa vote lagi, hasil final
}

/// Data kandidat
#[contracttype]
#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: u64,
    pub name: String,
    pub votes: u64,
}

/// Keys untuk menyimpan data di blockchain storage
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,              // Siapa admin-nya (Address)
    Status,             // Status voting (VotingStatus)
    Candidates,         // Daftar kandidat (Vec<Candidate>)
    HasVoted(Address),  // Apakah address ini sudah vote (bool)
    TotalVoters,        // Jumlah total pemilih (u64)
}

// ============================================================
//  CONTRACT
// ============================================================

#[contract]
pub struct SecureVotingContract;

#[contractimpl]
impl SecureVotingContract {

    // ========================================================
    //  INISIALISASI — Hanya bisa dipanggil 1 kali
    // ========================================================

    /// Inisialisasi contract. Orang yang memanggil ini jadi ADMIN.
    /// KEAMANAN: Fungsi ini hanya bisa dipanggil sekali.
    /// Setelah admin di-set, tidak bisa diubah lagi.
    pub fn initialize(env: Env, admin: Address) -> String {
        // Cek apakah sudah pernah di-inisialisasi
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract sudah diinisialisasi!");
        }

        // Verifikasi bahwa pemanggil memang pemilik address ini
        admin.require_auth();

        // Set admin
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Set status awal: Belum Dimulai
        env.storage()
            .instance()
            .set(&DataKey::Status, &VotingStatus::NotStarted);

        // Set total voters = 0
        env.storage()
            .instance()
            .set(&DataKey::TotalVoters, &0u64);

        String::from_str(&env, "Contract berhasil diinisialisasi")
    }

    // ========================================================
    //  FUNGSI ADMIN — Hanya admin yang bisa akses
    // ========================================================

    /// Tambah kandidat baru.
    /// KEAMANAN: Hanya admin, hanya saat voting belum dimulai.
    pub fn add_candidate(env: Env, admin: Address, name: String) -> String {
        // 1. Verifikasi admin
        Self::require_admin(&env, &admin);

        // 2. Pastikan voting belum dimulai
        let status = Self::get_status(env.clone());
        if status != VotingStatus::NotStarted {
            panic!("Tidak bisa tambah kandidat setelah voting dimulai!");
        }

        // 3. Tambah kandidat
        let mut candidates = Self::load_candidates(&env);
        let candidate = Candidate {
            id: env.prng().gen::<u64>(),
            name,
            votes: 0,
        };
        candidates.push_back(candidate);
        env.storage()
            .instance()
            .set(&DataKey::Candidates, &candidates);

        String::from_str(&env, "Kandidat berhasil ditambahkan")
    }

    /// Hapus kandidat.
    /// KEAMANAN: Hanya admin, hanya saat voting belum dimulai.
    pub fn remove_candidate(env: Env, admin: Address, candidate_id: u64) -> String {
        Self::require_admin(&env, &admin);

        let status = Self::get_status(env.clone());
        if status != VotingStatus::NotStarted {
            panic!("Tidak bisa hapus kandidat setelah voting dimulai!");
        }

        let mut candidates = Self::load_candidates(&env);
        for i in 0..candidates.len() {
            if candidates.get(i).unwrap().id == candidate_id {
                candidates.remove(i);
                env.storage()
                    .instance()
                    .set(&DataKey::Candidates, &candidates);
                return String::from_str(&env, "Kandidat dihapus");
            }
        }

        String::from_str(&env, "Kandidat tidak ditemukan")
    }

    /// Buka voting — setelah ini user bisa mulai vote.
    /// KEAMANAN: Hanya admin, harus ada minimal 2 kandidat.
    pub fn open_voting(env: Env, admin: Address) -> String {
        Self::require_admin(&env, &admin);

        let status = Self::get_status(env.clone());
        if status != VotingStatus::NotStarted {
            panic!("Voting sudah dimulai atau sudah selesai!");
        }

        // Pastikan ada minimal 2 kandidat
        let candidates = Self::load_candidates(&env);
        if candidates.len() < 2 {
            panic!("Minimal harus ada 2 kandidat untuk memulai voting!");
        }

        env.storage()
            .instance()
            .set(&DataKey::Status, &VotingStatus::Open);

        String::from_str(&env, "Voting dibuka!")
    }

    /// Tutup voting — setelah ini tidak ada yang bisa vote lagi.
    /// KEAMANAN: Hanya admin. Hasil menjadi final dan permanen.
    pub fn close_voting(env: Env, admin: Address) -> String {
        Self::require_admin(&env, &admin);

        let status = Self::get_status(env.clone());
        if status != VotingStatus::Open {
            panic!("Voting belum dibuka atau sudah ditutup!");
        }

        env.storage()
            .instance()
            .set(&DataKey::Status, &VotingStatus::Closed);

        String::from_str(&env, "Voting ditutup. Hasil bersifat final.")
    }

    // ========================================================
    //  FUNGSI VOTER — User yang mau vote
    // ========================================================

    /// Vote untuk kandidat tertentu.
    /// KEAMANAN:
    ///   - Voter harus autentikasi diri (require_auth)
    ///   - Hanya bisa vote 1 kali
    ///   - Hanya bisa saat voting sedang Open
    pub fn vote(env: Env, voter: Address, candidate_id: u64) -> String {
        // 1. Verifikasi identitas voter
        voter.require_auth();

        // 2. Pastikan voting sedang berlangsung
        let status = Self::get_status(env.clone());
        if status != VotingStatus::Open {
            panic!("Voting belum dibuka atau sudah ditutup!");
        }

        // 3. Cek apakah sudah pernah vote (anti double-voting)
        let voted_key = DataKey::HasVoted(voter.clone());
        let already_voted: bool = env
            .storage()
            .persistent()
            .get(&voted_key)
            .unwrap_or(false);

        if already_voted {
            panic!("Kamu sudah vote! Tidak bisa vote 2 kali.");
        }

        // 4. Cari kandidat dan tambah vote-nya
        let mut candidates = Self::load_candidates(&env);
        let mut found = false;

        for i in 0..candidates.len() {
            let c = candidates.get(i).unwrap();
            if c.id == candidate_id {
                let updated = Candidate {
                    id: c.id,
                    name: c.name,
                    votes: c.votes + 1,
                };
                candidates.set(i, updated);
                found = true;
                break;
            }
        }

        if !found {
            panic!("Kandidat tidak ditemukan!");
        }

        // 5. Simpan data
        env.storage()
            .instance()
            .set(&DataKey::Candidates, &candidates);

        // 6. Tandai voter sudah vote (PERMANEN, tidak bisa di-undo)
        env.storage().persistent().set(&voted_key, &true);

        // 7. Update total voters
        let total: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalVoters)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalVoters, &(total + 1));

        String::from_str(&env, "Vote berhasil dicatat!")
    }

    // ========================================================
    //  FUNGSI PUBLIK — Siapa saja bisa lihat
    // ========================================================

    /// Lihat semua kandidat beserta jumlah vote-nya
    pub fn get_candidates(env: Env) -> Vec<Candidate> {
        Self::load_candidates(&env)
    }

    /// Lihat status voting saat ini
    pub fn get_status(env: Env) -> VotingStatus {
        env.storage()
            .instance()
            .get(&DataKey::Status)
            .unwrap_or(VotingStatus::NotStarted)
    }

    /// Cek apakah seseorang sudah vote
    pub fn has_voted(env: Env, voter: Address) -> bool {
        let key = DataKey::HasVoted(voter);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    /// Lihat total pemilih yang sudah vote
    pub fn get_total_voters(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalVoters)
            .unwrap_or(0)
    }

    /// Lihat siapa admin contract ini
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract belum diinisialisasi!")
    }

    // ========================================================
    //  FUNGSI INTERNAL (Private) — Tidak bisa dipanggil dari luar
    // ========================================================

    /// Verifikasi bahwa pemanggil adalah admin.
    /// Kalau bukan admin, contract akan PANIC (gagal).
    fn require_admin(env: &Env, caller: &Address) {
        // 1. Pastikan caller memang pemilik address ini
        caller.require_auth();

        // 2. Ambil admin dari storage
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract belum diinisialisasi!");

        // 3. Bandingkan — kalau bukan admin, gagalkan transaksi
        if *caller != admin {
            panic!("Hanya admin yang bisa melakukan ini!");
        }
    }

    /// Helper: Load daftar kandidat dari storage
    fn load_candidates(env: &Env) -> Vec<Candidate> {
        env.storage()
            .instance()
            .get(&DataKey::Candidates)
            .unwrap_or(Vec::new(env))
    }
}

mod test;
