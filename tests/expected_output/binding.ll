; ModuleID = 'tests.binding'
source_filename = "tests.binding"

@string_literal = private unnamed_addr constant [5 x i8] c"test\00", align 1
@string_literal.2 = private unnamed_addr constant [3 x i8] c"hi\00", align 1

define private void @tests_binding.tests() {
fn.entry:
  %if.value = alloca i32, align 4
  br i1 true, label %if.then, label %if.else

if.after:                                         ; preds = %if.then, %if.else
  %access.if.value = load i32, ptr %if.value, align 4
  br i1 true, label %if.then2, label %if.after1

if.else:                                          ; preds = %fn.entry
  store i32 1, ptr %if.value, align 4
  br label %if.after

if.then:                                          ; preds = %fn.entry
  store i32 1, ptr %if.value, align 4
  br label %if.after

if.after1:                                        ; preds = %if.after, %if.then2
  call void (i32, ...) @emit(i32 1)
  call void (i32, ...) @emit(i32 1)
  call void (i32, ...) @emit(i32 2)
  call void (ptr, ...) @emit(ptr null)
  call void (ptr, ...) @emit(ptr null)
  call void (ptr, ...) @emit(ptr @string_literal.2)
  call void (i32, ...) @emit(i32 123)
  call void (i1, ...) @emit(i1 false)
  call void (ptr, ...) @emit(ptr @tests_binding.closure)
  call void (ptr, ...) @emit(ptr @tests_binding.closure.1)
  call void (i32, ...) @emit(i32 1)
  call void (i32, ...) @emit(i32 1)
  call void (ptr, ...) @emit(ptr null)
  ret void

if.then2:                                         ; preds = %if.after
  br label %if.after1
}

define private i32 @tests_binding.closure() {
closure.entry:
  ret i32 0
}

define private void @tests_binding.closure.1() {
closure.entry:
  ret void
}

declare void @emit(...)