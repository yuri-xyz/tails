; ModuleID = 'tests.as_int_to_real'
source_filename = "tests.as_int_to_real"

define private void @tests_as_int_to_real.tests() {
fn.entry:
  call void (half, ...) @emit(half 0xH57B0)
  call void (float, ...) @emit(float 1.230000e+02)
  call void (double, ...) @emit(double 1.230000e+02)
  call void (i32, ...) @emit(i32 123)
  call void (i16, ...) @emit(i16 123)
  call void (i8, ...) @emit(i8 123)
  ret void
}

declare void @emit(...)