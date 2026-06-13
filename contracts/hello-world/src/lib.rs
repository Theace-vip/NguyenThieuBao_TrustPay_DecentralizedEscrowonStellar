#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, symbol_short};

// Định nghĩa các trạng thái của một Job
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobStatus {
    Funded = 0,    // Đã nạp tiền vào contract
    Completed = 1, // Đã hoàn thành và chuyển tiền cho Freelancer
    Refunded = 2,  // Đã hoàn tiền lại cho Khách hàng
    Disputed = 3,  // Đang xảy ra tranh chấp
    Resolved = 4,  // Admin đã giải quyết tranh chấp xong
}

// Cấu trúc dữ liệu của một Job
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Job {
    pub owner: Address,     // Khách hàng
    pub worker: Address,    // Freelancer
    pub token: Address,     // Địa chỉ token thanh toán (ví dụ USDC/XLM)
    pub amount: i128,       // Số lượng token
    pub deadline: u64,      // Thời hạn hoàn thành (Timestamp)
    pub status: JobStatus,  // Trạng thái hiện tại
}

// Các key dùng để lưu trữ dữ liệu trong Soroban Storage
#[contracttype]
pub enum DataKey {
    Admin,       // Lưu địa chỉ Admin giải quyết tranh chấp
    JobCount,    // ID tự tăng cho Job
    Job(u64),    // Lưu thông tin Job theo ID
}

#[contract]
pub struct TrustPayContract;

#[contractimpl]
impl TrustPayContract {
    /// 1. Khởi tạo contract với địa chỉ Admin (người phân xử đa chữ ký nếu cần)
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::JobCount, &0u64);
    }

    /// 2. Khách hàng tạo job và nạp tiền (Deposit vào contract)
    pub fn create_job(
        env: Env,
        owner: Address,
        worker: Address,
        token: Address,
        amount: i128,
        deadline: u64,
    ) -> u64 {
        // Yêu cầu chữ ký của khách hàng
        owner.require_auth();

        if amount <= 0 {
            panic!("Amount must be greater than 0");
        }

        // Chuyển token từ ví Khách hàng vào Smart Contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&owner, &env.current_contract_address(), &amount);

        // Tăng ID Job
        let mut job_id: u64 = env.storage().instance().get(&DataKey::JobCount).unwrap_or(0);
        job_id += 1;
        env.storage().instance().set(&DataKey::JobCount, &job_id);

        // Tạo cấu trúc Job và lưu vào Persistent Storage
        let job = Job {
            owner: owner.clone(),
            worker: worker.clone(),
            token: token.clone(),
            amount,
            deadline,
            status: JobStatus::Funded,
        };
        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        // Bắn Event để UI lắng nghe
        env.events().publish((symbol_short!("created"), job_id), job);

        job_id
    }

    /// 3. Khách hàng xác nhận hoàn thành và giải phóng tiền cho Freelancer
    pub fn release_payment(env: Env, job_id: u64) {
        let mut job: Job = env.storage().persistent().get(&DataKey::Job(job_id)).expect("Job not found");
        
        // Chỉ Khách hàng mới có quyền release tiền
        job.owner.require_auth();

        if job.status != JobStatus::Funded {
            panic!("Job is not in Funded status");
        }

        // Chuyển tiền từ Contract cho Freelancer
        let token_client = token::Client::new(&env, &job.token);
        token_client.transfer(&env.current_contract_address(), &job.worker, &job.amount);

        // Cập nhật trạng thái
        job.status = JobStatus::Completed;
        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        env.events().publish((symbol_short!("released"), job_id), job_id);
    }

    /// 4. Hoàn tiền lại cho Khách hàng nếu quá hạn mà Freelancer chưa hoàn thành
    pub fn refund(env: Env, job_id: u64) {
        let mut job: Job = env.storage().persistent().get(&DataKey::Job(job_id)).expect("Job not found");
        
        // Yêu cầu chữ ký xác nhận của Khách hàng
        job.owner.require_auth();

        if job.status != JobStatus::Funded {
            panic!("Job is not in Funded status");
        }

        // Kiểm tra thời gian block hiện tại so với deadline
        let current_time = env.ledger().timestamp();
        if current_time <= job.deadline {
            panic!("Deadline has not passed yet");
        }

        // Trả lại tiền từ Contract cho Khách hàng
        let token_client = token::Client::new(&env, &job.token);
        token_client.transfer(&env.current_contract_address(), &job.owner, &job.amount);

        job.status = JobStatus::Refunded;
        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        env.events().publish((symbol_short!("refunded"), job_id), job_id);
    }

    /// 5. Báo cáo tranh chấp (Freelancer hoặc Khách hàng đều có thể gọi)
    pub fn raise_dispute(env: Env, caller: Address, job_id: u64) {
        caller.require_auth();

        let mut job: Job = env.storage().persistent().get(&DataKey::Job(job_id)).expect("Job not found");

        if caller != job.owner && caller != job.worker {
            panic!("Only owner or worker can raise a dispute");
        }

        if job.status != JobStatus::Funded {
            panic!("Can only dispute an active, funded job");
        }

        // Đóng băng trạng thái job thành Disputed
        job.status = JobStatus::Disputed;
        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        env.events().publish((symbol_short!("disputed"), job_id), job_id);
    }

    /// 6. Admin giải quyết tranh chấp và phân xử luồng tiền
    pub fn resolve_dispute(env: Env, job_id: u64, pay_to_worker: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        // Bắt buộc quyền Admin
        admin.require_auth();

        let mut job: Job = env.storage().persistent().get(&DataKey::Job(job_id)).expect("Job not found");

        if job.status != JobStatus::Disputed {
            panic!("Job is not in Disputed status");
        }

        let token_client = token::Client::new(&env, &job.token);

        // Quyết định tiền về tay ai
        if pay_to_worker {
            token_client.transfer(&env.current_contract_address(), &job.worker, &job.amount);
        } else {
            token_client.transfer(&env.current_contract_address(), &job.owner, &job.amount);
        }

        job.status = JobStatus::Resolved;
        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        env.events().publish((symbol_short!("resolved"), job_id), pay_to_worker);
    }

    /// 7. Hàm read-only để UI query thông tin Job
    pub fn get_job(env: Env, job_id: u64) -> Job {
        env.storage().persistent().get(&DataKey::Job(job_id)).expect("Job not found")
    }
}