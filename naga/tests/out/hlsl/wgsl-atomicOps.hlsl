struct Struct {
    uint atomic_scalar;
    int atomic_arr[2];
};

struct _atomic_compare_exchange_resultUint4_ {
    uint old_value;
    bool exchanged;
};

struct _atomic_compare_exchange_resultSint4_ {
    int old_value;
    bool exchanged;
};

RWByteAddressBuffer storage_atomic_scalar : register(u0);
RWByteAddressBuffer storage_atomic_arr : register(u1);
RWByteAddressBuffer storage_struct : register(u2);
groupshared uint workgroup_atomic_scalar;
groupshared int workgroup_atomic_arr[2];
groupshared Struct workgroup_struct;

[numthreads(2, 1, 1)]
void cs_main(uint3 id : SV_GroupThreadID, uint3 __local_invocation_id : SV_GroupThreadID)
{
    if (all(__local_invocation_id == uint3(0u, 0u, 0u))) {
        workgroup_atomic_scalar = (uint)0;
        workgroup_atomic_arr = (int[2])0;
        workgroup_struct = (Struct)0;
    }
    GroupMemoryBarrierWithGroupSync();
    storage_atomic_scalar.Store(0, asuint(1u));
    storage_atomic_arr.Store(4, asuint(int(1)));
    storage_struct.Store(0, asuint(1u));
    storage_struct.Store(4+4, asuint(int(1)));
    workgroup_atomic_scalar = 1u;
    workgroup_atomic_arr[1] = int(1);
    workgroup_struct.atomic_scalar = 1u;
    workgroup_struct.atomic_arr[1] = int(1);
    GroupMemoryBarrierWithGroupSync();
    uint l0_ = asuint(storage_atomic_scalar.Load(0));
    int l1_ = asint(storage_atomic_arr.Load(4));
    uint l2_ = asuint(storage_struct.Load(0));
    int l3_ = asint(storage_struct.Load(4+4));
    uint l4_ = workgroup_atomic_scalar;
    int l5_ = workgroup_atomic_arr[1];
    uint l6_ = workgroup_struct.atomic_scalar;
    int l7_ = workgroup_struct.atomic_arr[1];
    GroupMemoryBarrierWithGroupSync();
    uint _e51; storage_atomic_scalar.InterlockedAdd(0, 1u, _e51);
    int _e55; storage_atomic_arr.InterlockedAdd(4, int(1), _e55);
    uint _e59; storage_struct.InterlockedAdd(0, 1u, _e59);
    int _e64; storage_struct.InterlockedAdd(4+4, int(1), _e64);
    uint _e67; InterlockedAdd(workgroup_atomic_scalar, 1u, _e67);
    int _e71; InterlockedAdd(workgroup_atomic_arr[1], int(1), _e71);
    uint _e75; InterlockedAdd(workgroup_struct.atomic_scalar, 1u, _e75);
    int _e80; InterlockedAdd(workgroup_struct.atomic_arr[1], int(1), _e80);
    GroupMemoryBarrierWithGroupSync();
    uint _e83; storage_atomic_scalar.InterlockedAdd(0, -1u, _e83);
    int _e87; storage_atomic_arr.InterlockedAdd(4, -int(1), _e87);
    uint _e91; storage_struct.InterlockedAdd(0, -1u, _e91);
    int _e96; storage_struct.InterlockedAdd(4+4, -int(1), _e96);
    uint _e99; InterlockedAdd(workgroup_atomic_scalar, -1u, _e99);
    int _e103; InterlockedAdd(workgroup_atomic_arr[1], -int(1), _e103);
    uint _e107; InterlockedAdd(workgroup_struct.atomic_scalar, -1u, _e107);
    int _e112; InterlockedAdd(workgroup_struct.atomic_arr[1], -int(1), _e112);
    GroupMemoryBarrierWithGroupSync();
    uint _e115; storage_atomic_scalar.InterlockedMax(0, 1u, _e115);
    int _e119; storage_atomic_arr.InterlockedMax(4, int(1), _e119);
    uint _e123; storage_struct.InterlockedMax(0, 1u, _e123);
    int _e128; storage_struct.InterlockedMax(4+4, int(1), _e128);
    uint _e131; InterlockedMax(workgroup_atomic_scalar, 1u, _e131);
    int _e135; InterlockedMax(workgroup_atomic_arr[1], int(1), _e135);
    uint _e139; InterlockedMax(workgroup_struct.atomic_scalar, 1u, _e139);
    int _e144; InterlockedMax(workgroup_struct.atomic_arr[1], int(1), _e144);
    GroupMemoryBarrierWithGroupSync();
    uint _e147; storage_atomic_scalar.InterlockedMin(0, 1u, _e147);
    int _e151; storage_atomic_arr.InterlockedMin(4, int(1), _e151);
    uint _e155; storage_struct.InterlockedMin(0, 1u, _e155);
    int _e160; storage_struct.InterlockedMin(4+4, int(1), _e160);
    uint _e163; InterlockedMin(workgroup_atomic_scalar, 1u, _e163);
    int _e167; InterlockedMin(workgroup_atomic_arr[1], int(1), _e167);
    uint _e171; InterlockedMin(workgroup_struct.atomic_scalar, 1u, _e171);
    int _e176; InterlockedMin(workgroup_struct.atomic_arr[1], int(1), _e176);
    GroupMemoryBarrierWithGroupSync();
    uint _e179; storage_atomic_scalar.InterlockedAnd(0, 1u, _e179);
    int _e183; storage_atomic_arr.InterlockedAnd(4, int(1), _e183);
    uint _e187; storage_struct.InterlockedAnd(0, 1u, _e187);
    int _e192; storage_struct.InterlockedAnd(4+4, int(1), _e192);
    uint _e195; InterlockedAnd(workgroup_atomic_scalar, 1u, _e195);
    int _e199; InterlockedAnd(workgroup_atomic_arr[1], int(1), _e199);
    uint _e203; InterlockedAnd(workgroup_struct.atomic_scalar, 1u, _e203);
    int _e208; InterlockedAnd(workgroup_struct.atomic_arr[1], int(1), _e208);
    GroupMemoryBarrierWithGroupSync();
    uint _e211; storage_atomic_scalar.InterlockedOr(0, 1u, _e211);
    int _e215; storage_atomic_arr.InterlockedOr(4, int(1), _e215);
    uint _e219; storage_struct.InterlockedOr(0, 1u, _e219);
    int _e224; storage_struct.InterlockedOr(4+4, int(1), _e224);
    uint _e227; InterlockedOr(workgroup_atomic_scalar, 1u, _e227);
    int _e231; InterlockedOr(workgroup_atomic_arr[1], int(1), _e231);
    uint _e235; InterlockedOr(workgroup_struct.atomic_scalar, 1u, _e235);
    int _e240; InterlockedOr(workgroup_struct.atomic_arr[1], int(1), _e240);
    GroupMemoryBarrierWithGroupSync();
    uint _e243; storage_atomic_scalar.InterlockedXor(0, 1u, _e243);
    int _e247; storage_atomic_arr.InterlockedXor(4, int(1), _e247);
    uint _e251; storage_struct.InterlockedXor(0, 1u, _e251);
    int _e256; storage_struct.InterlockedXor(4+4, int(1), _e256);
    uint _e259; InterlockedXor(workgroup_atomic_scalar, 1u, _e259);
    int _e263; InterlockedXor(workgroup_atomic_arr[1], int(1), _e263);
    uint _e267; InterlockedXor(workgroup_struct.atomic_scalar, 1u, _e267);
    int _e272; InterlockedXor(workgroup_struct.atomic_arr[1], int(1), _e272);
    uint _e275; storage_atomic_scalar.InterlockedExchange(0, 1u, _e275);
    int _e279; storage_atomic_arr.InterlockedExchange(4, int(1), _e279);
    uint _e283; storage_struct.InterlockedExchange(0, 1u, _e283);
    int _e288; storage_struct.InterlockedExchange(4+4, int(1), _e288);
    uint _e291; InterlockedExchange(workgroup_atomic_scalar, 1u, _e291);
    int _e295; InterlockedExchange(workgroup_atomic_arr[1], int(1), _e295);
    uint _e299; InterlockedExchange(workgroup_struct.atomic_scalar, 1u, _e299);
    int _e304; InterlockedExchange(workgroup_struct.atomic_arr[1], int(1), _e304);
    _atomic_compare_exchange_resultUint4_ _e308; storage_atomic_scalar.InterlockedCompareExchange(0, 1u, 2u, _e308.old_value);
    _e308.exchanged = (_e308.old_value == 1u);
    _atomic_compare_exchange_resultSint4_ _e313; storage_atomic_arr.InterlockedCompareExchange(4, int(1), int(2), _e313.old_value);
    _e313.exchanged = (_e313.old_value == int(1));
    _atomic_compare_exchange_resultUint4_ _e318; storage_struct.InterlockedCompareExchange(0, 1u, 2u, _e318.old_value);
    _e318.exchanged = (_e318.old_value == 1u);
    _atomic_compare_exchange_resultSint4_ _e324; storage_struct.InterlockedCompareExchange(4+4, int(1), int(2), _e324.old_value);
    _e324.exchanged = (_e324.old_value == int(1));
    _atomic_compare_exchange_resultUint4_ _e328; InterlockedCompareExchange(workgroup_atomic_scalar, 1u, 2u, _e328.old_value);
    _e328.exchanged = (_e328.old_value == 1u);
    _atomic_compare_exchange_resultSint4_ _e333; InterlockedCompareExchange(workgroup_atomic_arr[1], int(1), int(2), _e333.old_value);
    _e333.exchanged = (_e333.old_value == int(1));
    _atomic_compare_exchange_resultUint4_ _e338; InterlockedCompareExchange(workgroup_struct.atomic_scalar, 1u, 2u, _e338.old_value);
    _e338.exchanged = (_e338.old_value == 1u);
    _atomic_compare_exchange_resultSint4_ _e344; InterlockedCompareExchange(workgroup_struct.atomic_arr[1], int(1), int(2), _e344.old_value);
    _e344.exchanged = (_e344.old_value == int(1));
    return;
}
