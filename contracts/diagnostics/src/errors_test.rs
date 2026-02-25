use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TestError {
    Test1 = 1,
    Test2 = 2,
}
