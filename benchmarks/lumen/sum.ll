; ModuleID = 'lumen'
source_filename = "lumen.nv"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"

declare i64 @_lw_int(i64)
declare i64 @_lw_flt(double)
declare i64 @_lw_bool(i64)
declare i64 @_lw_str(i64)
declare i64 @_lw_void()
declare i64 @_lw_none()
declare void @_lw_print(i64)
declare void @_lw_print_blank()
declare i64 @_lw_join(i64, i64)
declare i64 @_lw_bin(i64, i64, i64)
declare i64 @_lw_un(i64, i64)
declare i64 @_lw_truthy_i(i64)
declare i64 @_lw_arr_new()
declare i64 @_lw_arr_push(i64, i64)
declare i64 @_lw_arr_get(i64, i64)
declare i64 @_lw_arr_set(i64, i64, i64)
declare i64 @_lw_arr_len(i64)
declare i64 @_lw_arr_rev(i64)
declare i64 @_lw_arr_sort(i64)
declare i64 @_lw_st_new()
declare i64 @_lw_st_add(i64, i64, i64)
declare i64 @_lw_st_get(i64, i64)
declare i64 @_lw_st_set(i64, i64, i64)
declare i64 @_lw_tup_new()
declare i64 @_lw_tup_push(i64, i64)
declare i64 @_lw_tup_get(i64, i64)
declare i64 @_lw_read()
declare i64 @_lw_typeof(i64)
declare i64 @_lw_to_text(i64)
declare i64 @_lw_sub(i64, i64, i64)
declare i64 @_lw_concat_list(i64)
declare i64 @_lw_some(i64)
declare i64 @_lw_ok(i64)
declare i64 @_lw_err(i64)
declare i64 @_lw_map_new()
declare i64 @_lw_map_set(i64, i64, i64)
declare i64 @_lw_map_get(i64, i64)
declare i64 @_lw_map_has(i64, i64)
declare i64 @_lw_map_len(i64)
declare i64 @_lw_map_keys(i64)
declare void @_lw_try_begin()
declare void @_lw_try_end()
declare i64 @_lw_err_active()
declare i64 @_lw_err_take()
declare i64 @_lw_kind(i64)
declare i64 @_lw_payload(i64)
declare i64 @_lw_enm_new(i64, i64, i64)
declare i64 @_lw_enm_variant_is(i64, i64)
declare i64 @_lw_fref(i64, i64)
declare i64 @_lw_fref_addr(i64)
declare i64 @_lw_mkref(i64)
declare i64 @_lw_load_slot(i64)
declare void @_lw_store_slot(i64, i64)
declare void @_lw_store_slot_direct(i64, i64)
declare i64 @_lw_dcp(i64)
declare i64 @_lw_arr_push_ip(i64, i64)
declare i64 @_lw_abs(i64)
declare i64 @_lw_sqrt(i64)
declare i64 @_lw_pow(i64, i64)
declare i64 @_lw_floor(i64)
declare i64 @_lw_ceil(i64)
declare i64 @_lw_round(i64)

@lw_str_0 = private unnamed_addr constant [5 x i8] c"sum:\00"

define i64 @lum___main__() {
entry:
  %r0 = call i64 @_lw_void()
  %r1 = getelementptr [5 x i8], [5 x i8]* @lw_str_0, i64 0, i64 0
  %r2 = ptrtoint i8* %r1 to i64
  %r3 = call i64 @_lw_str(i64 %r2)
  %r4 = call i64 @_lw_int(i64 10000000)
  %r5 = call i64 @_lw_dcp(i64 %r4)
  %r6 = call i64 @lum_suma(i64 %r5)
  %r7 = call i64 @_lw_err_active()
  %r8 = icmp ne i64 %r7, 0
  br i1 %r8, label %ec_d_9, label %ec_o_10
ec_d_9:
  %r11 = call i64 @_lw_void()
  ret i64 %r11
ec_o_10:
  %r12 = call i64 @_lw_bin(i64 1, i64 %r3, i64 %r6)
  %r13 = call i64 @_lw_err_active()
  %r14 = icmp ne i64 %r13, 0
  br i1 %r14, label %ec_d_15, label %ec_o_16
ec_d_15:
  %r17 = call i64 @_lw_void()
  ret i64 %r17
ec_o_16:
  call void @_lw_print(i64 %r12)
  %r18 = call i64 @_lw_void()
  ret i64 %r18
}

define i64 @lum_suma(i64 %p0) {
entry:
  %cell_n = alloca [80 x i8]
  %r0 = call i64 @_lw_void()
  %r1 = ptrtoint [80 x i8]* %cell_n to i64
  call void @_lw_store_slot_direct(i64 %r1, i64 %r0)
  %r2 = ptrtoint [80 x i8]* %cell_n to i64
  call void @_lw_store_slot_direct(i64 %r2, i64 %p0)
  %var_acc = alloca i64
  store i64 %r0, i64* %var_acc
  %var_i = alloca i64
  store i64 %r0, i64* %var_i
  %r3 = call i64 @_lw_int(i64 0)
  %r4 = call i64 @_lw_dcp(i64 %r3)
  store i64 %r4, i64* %var_acc
  %r5 = call i64 @_lw_int(i64 0)
  %r6 = call i64 @_lw_dcp(i64 %r5)
  store i64 %r6, i64* %var_i
  br label %L_0
L_0:
  %r8 = load i64, i64* %var_i
  %r10 = ptrtoint [80 x i8]* %cell_n to i64
  %r11 = call i64 @_lw_load_slot(i64 %r10)
  %r12 = call i64 @_lw_bin(i64 9, i64 %r8, i64 %r11)
  %r13 = call i64 @_lw_err_active()
  %r14 = icmp ne i64 %r13, 0
  br i1 %r14, label %ec_d_15, label %ec_o_16
ec_d_15:
  %r17 = call i64 @_lw_void()
  ret i64 %r17
ec_o_16:
  %r18 = call i64 @_lw_truthy_i(i64 %r12)
  %r19 = icmp eq i64 %r18, 0
  br i1 %r19, label %L_1, label %jf_20
jf_20:
  %r22 = load i64, i64* %var_acc
  %r24 = load i64, i64* %var_i
  %r25 = call i64 @_lw_bin(i64 1, i64 %r22, i64 %r24)
  %r26 = call i64 @_lw_err_active()
  %r27 = icmp ne i64 %r26, 0
  br i1 %r27, label %ec_d_28, label %ec_o_29
ec_d_28:
  %r30 = call i64 @_lw_void()
  ret i64 %r30
ec_o_29:
  %r31 = call i64 @_lw_dcp(i64 %r25)
  store i64 %r31, i64* %var_acc
  %r33 = load i64, i64* %var_i
  %r34 = call i64 @_lw_int(i64 1)
  %r35 = call i64 @_lw_bin(i64 1, i64 %r33, i64 %r34)
  %r36 = call i64 @_lw_err_active()
  %r37 = icmp ne i64 %r36, 0
  br i1 %r37, label %ec_d_38, label %ec_o_39
ec_d_38:
  %r40 = call i64 @_lw_void()
  ret i64 %r40
ec_o_39:
  %r41 = call i64 @_lw_dcp(i64 %r35)
  store i64 %r41, i64* %var_i
  br label %L_0
L_1:
  %r43 = load i64, i64* %var_acc
  ret i64 %r43
}

define i32 @main() {
entry:
  %res = call i64 @lum___main__()
  ret i32 0
}
