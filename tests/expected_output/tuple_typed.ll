; ModuleID = 'tests.tuple_typed'
source_filename = "tests.tuple_typed"

@string_literal = private unnamed_addr constant [5 x i8] c"test\00", align 1

define private void @tests_tuple_typed.tests() {
fn.entry:
  %tuple.alloca6 = alloca { i32, ptr }, align 8
  %tuple.alloca4 = alloca { i32 }, align 8
  %tuple.alloca1 = alloca { i32, ptr }, align 8
  %tuple.alloca = alloca { i32 }, align 8
  %tuple.init.gep = getelementptr inbounds { i32 }, ptr %tuple.alloca, i32 0, i32 0
  store i32 0, ptr %tuple.init.gep, align 4
  %tuple.init.gep2 = getelementptr inbounds { i32, ptr }, ptr %tuple.alloca1, i32 0, i32 0
  store i32 1, ptr %tuple.init.gep2, align 4
  %tuple.init.gep3 = getelementptr inbounds { i32, ptr }, ptr %tuple.alloca1, i32 0, i32 1
  store ptr @string_literal, ptr %tuple.init.gep3, align 8
  %tuple.init.gep5 = getelementptr inbounds { i32 }, ptr %tuple.alloca4, i32 0, i32 0
  store i32 0, ptr %tuple.init.gep5, align 4
  %access.tuple.access = load { i32 }, ptr %tuple.alloca4, align 4
  call void ({ i32 }, ...) @emit({ i32 } %access.tuple.access)
  %tuple.init.gep7 = getelementptr inbounds { i32, ptr }, ptr %tuple.alloca6, i32 0, i32 0
  store i32 1, ptr %tuple.init.gep7, align 4
  %tuple.init.gep8 = getelementptr inbounds { i32, ptr }, ptr %tuple.alloca6, i32 0, i32 1
  store ptr @string_literal, ptr %tuple.init.gep8, align 8
  %access.tuple.access9 = load { i32, ptr }, ptr %tuple.alloca6, align 8
  call void ({ i32, ptr }, ...) @emit({ i32, ptr } %access.tuple.access9)
  ret void
}

declare void @emit(...)