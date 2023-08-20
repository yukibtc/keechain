from keechain import Seed

mnemonic = "era know jaguar list tooth gravity eternal uphold deputy rural rebuild candy violin medal virtual noodle fix program fault stadium ceiling robot much zero"
seed = Seed.from_mnemonic(mnemonic)
print(seed.to_hex())