# Solores Makefile
# 用于构建、测试和批量生成IDL接口

# 变量定义
SOLORES_BIN := ./target/release/solores
WRAPPER_BIN := ./scripts/solores-wrapper.py
CARGO := cargo
TEST_OUTPUT := test_output
IDL_DIR := idls
BATCH_OUTPUT := batch_output_all_interfaces

# 关键IDL文件列表（包含各种格式的代表性IDL）
KEY_IDLS := raydium_launchpad pump_amm raydium serum phoenix moonshot

# SPL和Native程序IDL列表
SPL_IDLS := spl-token spl-token-2022 memo
NATIVE_IDLS := system compute-budget

# 颜色定义
GREEN := \033[0;32m
RED := \033[0;31m
BLUE := \033[0;34m
CYAN := \033[0;36m
MAGENTA := \033[0;35m
YELLOW := \033[0;33m
NC := \033[0m # No Color

# 默认目标
all: build

# 构建solores
build:
	@echo "$(BLUE)🔨 构建Solores...$(NC)"
	@$(CARGO) build --release
	@echo "$(GREEN)✅ 构建完成$(NC)"

# 快速测试关键IDL
test: build
	@echo "$(BLUE)🧪 测试关键IDL文件...$(NC)"
	@echo "================================"
	@mkdir -p $(TEST_OUTPUT)/test_makefile
	@total=0; passed=0; \
	for idl in $(KEY_IDLS); do \
		total=$$(($$total + 1)); \
		echo "$(BLUE)测试 $$idl.json...$(NC)"; \
		$(SOLORES_BIN) $(IDL_DIR)/$$idl.json -o $(TEST_OUTPUT)/test_makefile/$$idl --generate-parser > /dev/null 2>&1; \
		if [ $$? -eq 0 ]; then \
			echo "  $(GREEN)✅ 生成成功$(NC)"; \
			project_dir=$$(find $(TEST_OUTPUT)/test_makefile/$$idl -name "Cargo.toml" -type f | head -1 | xargs dirname); \
			if [ -n "$$project_dir" ]; then \
				cd "$$project_dir" && cargo check --features serde > /dev/null 2>&1; \
				if [ $$? -eq 0 ]; then \
					echo "  $(GREEN)✅ serde编译成功$(NC)"; \
					passed=$$(($$passed + 1)); \
				else \
					echo "  $(RED)❌ serde编译失败$(NC)"; \
				fi; \
				cd - > /dev/null; \
			else \
				echo "  $(RED)❌ 找不到项目目录$(NC)"; \
			fi; \
		else \
			echo "  $(RED)❌ 生成失败$(NC)"; \
		fi; \
		echo ""; \
	done; \
	echo "================================"; \
	echo "$(BLUE)测试报告: $$passed/$$total 通过$(NC)"; \
	if [ $$passed -eq $$total ]; then \
		echo "$(GREEN)🎉 所有测试通过！$(NC)"; \
	else \
		echo "$(RED)⚠️ 有测试失败，请检查输出$(NC)"; \
	fi

# 测试单个IDL
test-one:
	@if [ -z "$(IDL)" ]; then \
		echo "$(RED)错误: 未指定IDL文件$(NC)"; \
		echo "用法: make test-one IDL=raydium_launchpad"; \
		exit 1; \
	fi
	@echo "$(BLUE)测试 $(IDL).json...$(NC)"
	@mkdir -p $(TEST_OUTPUT)/test_single
	@$(SOLORES_BIN) $(IDL_DIR)/$(IDL).json -o $(TEST_OUTPUT)/test_single/$(IDL) --generate-parser
	@project_dir=$$(find $(TEST_OUTPUT)/test_single/$(IDL) -name "Cargo.toml" -type f | head -1 | xargs dirname); \
	if [ -n "$$project_dir" ]; then \
		cd "$$project_dir" && cargo check --features serde; \
	else \
		echo "$(RED)❌ 找不到项目目录$(NC)"; \
		exit 1; \
	fi

# 批量生成所有IDL
batch: build
	@echo "$(BLUE)📦 批量生成所有IDL...$(NC)"
	@$(SOLORES_BIN) $(IDL_DIR)/ --batch --generate-parser --batch-output-dir $(BATCH_OUTPUT)/
	@echo "$(GREEN)✅ 批量生成完成$(NC)"

# 生成到指定目录
generate-to:
	@if [ -z "$(OUTPUT_DIR)" ]; then \
		echo "$(RED)错误: 未指定输出目录$(NC)"; \
		echo "用法: make generate-to OUTPUT_DIR=/path/to/output"; \
		exit 1; \
	fi
	@echo "$(BLUE)📦 批量生成到 $(OUTPUT_DIR)...$(NC)"
	@$(SOLORES_BIN) $(IDL_DIR)/ --batch --generate-parser --batch-output-dir $(OUTPUT_DIR)/
	@echo "$(GREEN)✅ 生成完成$(NC)"

# 生成到arbitrage项目（快捷方式）
generate-arbitrage:
	@echo "$(BLUE)📦 生成到solana-arbitrage项目...$(NC)"
	@$(SOLORES_BIN) /Users/pingzi/Developer/work/solana/solana-arbitrage/idls/json/ --batch --generate-parser --batch-output-dir /Users/pingzi/Developer/work/solana/solana-arbitrage/idls/interfaces/
	@echo "$(GREEN)✅ 生成完成$(NC)"

# 清理测试文件
clean:
	@echo "$(BLUE)🧹 清理测试文件...$(NC)"
	@rm -rf $(TEST_OUTPUT)/test_makefile
	@rm -rf $(TEST_OUTPUT)/test_single
	@rm -rf $(TEST_OUTPUT)/serde_verify_*
	@echo "$(GREEN)✅ 清理完成$(NC)"

# 深度清理（包括所有test_output）
clean-all:
	@echo "$(BLUE)🧹 深度清理...$(NC)"
	@rm -rf $(TEST_OUTPUT)
	@rm -rf $(BATCH_OUTPUT)
	@rm -rf batch_output_*
	@echo "$(GREEN)✅ 深度清理完成$(NC)"

# 运行Rust测试
test-rust:
	@echo "$(BLUE)🦀 运行Rust测试...$(NC)"
	@$(CARGO) test

# 检查代码
check:
	@echo "$(BLUE)🔍 检查代码...$(NC)"
	@$(CARGO) check

# 检查serde特性
check-serde: build
	@echo "$(BLUE)🔍 检查serde特性支持...$(NC)"
	@echo "生成测试项目..."
	@$(SOLORES_BIN) $(IDL_DIR)/raydium_launchpad.json -o $(TEST_OUTPUT)/check_serde --generate-parser > /dev/null 2>&1
	@project_dir=$$(find $(TEST_OUTPUT)/check_serde -name "Cargo.toml" -type f | head -1 | xargs dirname); \
	if [ -n "$$project_dir" ]; then \
		cd "$$project_dir" && cargo check --features serde > /dev/null 2>&1; \
		if [ $$? -eq 0 ]; then \
			echo "$(GREEN)✅ serde特性支持正常$(NC)"; \
		else \
			echo "$(RED)❌ serde特性编译失败$(NC)"; \
			exit 1; \
		fi; \
	fi

# 显示IDL列表
list-idls:
	@echo "$(BLUE)📋 可用的IDL文件:$(NC)"
	@for idl in $(IDL_DIR)/*.json; do \
		basename $$idl .json; \
	done


# ==================== 批量处理过滤功能 ====================

# 批量生成时排除指定IDL
batch-exclude: build
	@if [ -z "$(EXCLUDE)" ]; then \
		echo "$(YELLOW)⚠️  未指定排除文件，使用默认批量生成$(NC)"; \
		$(MAKE) batch; \
	else \
		echo "$(CYAN)📦 批量生成（排除: $(EXCLUDE)）...$(NC)"; \
		$(SOLORES_BIN) $(IDL_DIR)/ --batch --generate-parser --batch-exclude "$(EXCLUDE)" --batch-output-dir $(TEST_OUTPUT)/batch_exclude/; \
		echo "$(GREEN)✅ 批量生成完成（已排除指定文件）$(NC)"; \
	fi

# 批量生成时仅包含指定IDL
batch-include: build
	@if [ -z "$(INCLUDE)" ]; then \
		echo "$(RED)错误: 未指定包含文件$(NC)"; \
		echo "用法: make batch-include INCLUDE='raydium*.json,pump*.json'"; \
		exit 1; \
	fi
	@echo "$(CYAN)📦 批量生成（仅包含: $(INCLUDE)）...$(NC)"
	@$(SOLORES_BIN) $(IDL_DIR)/ --batch --generate-parser --batch-include "$(INCLUDE)" --batch-output-dir $(TEST_OUTPUT)/batch_include/
	@echo "$(GREEN)✅ 批量生成完成（仅包含指定文件）$(NC)"

# ==================== Serde 特性相关 ====================

# 测试所有关键IDL的serde特性
test-serde: build
	@echo "$(YELLOW)🔍 测试所有关键IDL的serde特性...$(NC)"
	@echo "================================"
	@mkdir -p $(TEST_OUTPUT)/test_serde
	@total=0; passed=0; \
	for idl in $(KEY_IDLS); do \
		total=$$(($$total + 1)); \
		echo "$(YELLOW)测试 $$idl.json serde特性...$(NC)"; \
		$(SOLORES_BIN) $(IDL_DIR)/$$idl.json -o $(TEST_OUTPUT)/test_serde/$$idl --generate-parser > /dev/null 2>&1; \
		if [ $$? -eq 0 ]; then \
			project_dir=$$(find $(TEST_OUTPUT)/test_serde/$$idl -name "Cargo.toml" -type f | head -1 | xargs dirname); \
			if [ -n "$$project_dir" ]; then \
				cd "$$project_dir" && cargo test --features serde > /dev/null 2>&1; \
				if [ $$? -eq 0 ]; then \
					echo "  $(GREEN)✅ serde测试通过$(NC)"; \
					passed=$$(($$passed + 1)); \
				else \
					echo "  $(RED)❌ serde测试失败$(NC)"; \
				fi; \
				cd - > /dev/null; \
			fi; \
		else \
			echo "  $(RED)❌ 生成失败$(NC)"; \
		fi; \
	done; \
	echo "================================"; \
	echo "$(YELLOW)Serde测试报告: $$passed/$$total 通过$(NC)"

# 测试单个IDL的serde特性
test-serde-one:
	@if [ -z "$(IDL)" ]; then \
		echo "$(RED)错误: 未指定IDL文件$(NC)"; \
		echo "用法: make test-serde-one IDL=raydium_launchpad"; \
		exit 1; \
	fi
	@echo "$(YELLOW)🔍 测试 $(IDL).json 的serde特性...$(NC)"
	@mkdir -p $(TEST_OUTPUT)/test_serde_single
	@$(SOLORES_BIN) $(IDL_DIR)/$(IDL).json -o $(TEST_OUTPUT)/test_serde_single/$(IDL) --generate-parser
	@project_dir=$$(find $(TEST_OUTPUT)/test_serde_single/$(IDL) -name "Cargo.toml" -type f | head -1 | xargs dirname); \
	if [ -n "$$project_dir" ]; then \
		echo "运行serde测试..."; \
		cd "$$project_dir" && cargo test --features serde; \
	fi

# 批量生成并验证serde特性
batch-serde: build
	@echo "$(YELLOW)📦 批量生成并验证serde特性...$(NC)"
	@$(SOLORES_BIN) $(IDL_DIR)/ --batch --generate-parser --batch-output-dir $(TEST_OUTPUT)/batch_serde/
	@echo "$(GREEN)✅ 批量生成完成，开始验证serde...$(NC)"
	@failed=0; \
	for dir in $(TEST_OUTPUT)/batch_serde/*/; do \
		if [ -f "$$dir/Cargo.toml" ]; then \
			name=$$(basename $$dir); \
			echo -n "验证 $$name... "; \
			cd "$$dir" && cargo check --features serde > /dev/null 2>&1; \
			if [ $$? -eq 0 ]; then \
				echo "$(GREEN)✅$(NC)"; \
			else \
				echo "$(RED)❌$(NC)"; \
				failed=$$(($$failed + 1)); \
			fi; \
			cd - > /dev/null; \
		fi; \
	done; \
	if [ $$failed -eq 0 ]; then \
		echo "$(GREEN)🎉 所有项目serde特性验证通过！$(NC)"; \
	else \
		echo "$(RED)⚠️  有 $$failed 个项目serde验证失败$(NC)"; \
	fi

# 帮助信息
help:
	@echo "$(BLUE)Solores Makefile 使用指南$(NC)"
	@echo ""
	@echo "$(GREEN)基础命令:$(NC)"
	@echo "  make build         - 构建solores工具"
	@echo "  make test          - 测试关键IDL文件 ($(KEY_IDLS))"
	@echo "  make test-one IDL=<name> - 测试单个IDL文件"
	@echo "  make batch         - 批量生成所有IDL"
	@echo "  make clean         - 清理测试文件"
	@echo "  make clean-all     - 深度清理所有生成文件"
	@echo ""
	@echo "$(CYAN)批量处理过滤:$(NC)"
	@echo "  make batch-exclude EXCLUDE='pump*.json' - 排除指定文件"
	@echo "  make batch-include INCLUDE='raydium*.json' - 仅包含指定文件"
	@echo ""
	@echo "$(YELLOW)Serde特性:$(NC)"
	@echo "  make test-serde    - 测试所有关键IDL的serde特性"
	@echo "  make test-serde-one IDL=<name> - 测试单个IDL的serde特性"
	@echo "  make batch-serde   - 批量生成并验证serde特性"
	@echo "  make check-serde   - 检查serde特性支持"
	@echo ""
	@echo "$(GREEN)其他命令:$(NC)"
	@echo "  make generate-to OUTPUT_DIR=<path> - 生成到指定目录"
	@echo "  make generate-arbitrage - 生成到solana-arbitrage项目"
	@echo "  make list-idls     - 显示可用的IDL文件"
	@echo "  make test-rust     - 运行Rust测试"
	@echo "  make check         - 检查代码"
	@echo "  make help          - 显示此帮助信息"
	@echo ""
	@echo "$(BLUE)示例:$(NC)"
	@echo "  make test"
	@echo "  make test-one IDL=raydium"
	@echo "  make batch-exclude EXCLUDE='*.json,!raydium*.json'"
	@echo "  make batch-include INCLUDE='pump*.json,phoenix.json'"
	@echo "  make test-serde-one IDL=serum"

.PHONY: all build test test-one batch generate-to generate-arbitrage clean clean-all test-rust check check-serde list-idls \
        batch-exclude batch-include \
        test-serde test-serde-one batch-serde \
        help