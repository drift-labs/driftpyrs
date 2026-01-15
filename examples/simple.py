import driftpy
from driftpyrs import get_vault_program_id

if __name__ == "__main__":
    print("Vault program ID:", get_vault_program_id())
    print("driftpy version:", driftpy.__version__)
