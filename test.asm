TAG_SIZE=8
TAG_BITMASK=0xff
RECORD_SIZE=31
BLOCK_SIZE=8

# key is in format :

# rotr(r, bs) = ((bs << r) | (bs >> (BLOCK_SIZE - r))) & ((1 << BLOCK_SIZE) - 1)

# flip(bf) = ~bf
# shift(r, bs) = ((bs << r) | (bs >> (BLOCK_SIZE - r))) & ((1 << BLOCK_SIZE) - 1)
# swap(bx, by, px, py, s) = 
#   seg1 = shift(bx, px) && 

validate_record:
    # Record in r8
    li  r8, 0 # 
    li  r9, 0 # block shift amt
    li  r15, TAG_BITMASK

    shr r10, r8, r9
    and r10, r10, r15

    # block is now in r10
    

    add r9, TAG_SIZE

init_tallies:
    li  r1, 0 # Candidate 1
    li  r2, 0 # Candidate 2
    li  r3, 0 # Candidate 3
    li  r4, 0 # Candidate 4
