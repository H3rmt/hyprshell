SHELL := /bin/bash

.PHONY: all clean combine prepare-combined copy-defs build build-combined test test-combined

SRCS_DIR := ./src
SRCS_FILES := key-press.cpp layer-change.cpp init.cpp exit.cpp main.cpp send.cpp mouse-button.cpp keyboard-focus.cpp
SRCS := $(addprefix $(SRCS_DIR)/,$(SRCS_FILES))

PREPAREDIR := out
OUTNAME := hyprshell.so
TARGET := $(OUTNAME)

HEADERS := globals.h handlers.h defs.h send.h

CXX := g++
CXXFLAGS := -fPIC -std=c++2b -O2 -I/usr/include/pixman-1
LDFLAGS := -shared --no-gnu-unique

all: build

# Build by compiling object files then linking into a shared object.
OBJ := $(patsubst $(SRCS_DIR)/%.cpp,$(PREPAREDIR)/%.o,$(SRCS))

build: $(TARGET)

$(TARGET): $(OBJ)
	$(CXX) $(LDFLAGS) -o $@ $^

$(PREPAREDIR)/%.o: $(SRCS_DIR)/%.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Prepare a single combined source file in $(PREPAREDIR)
prepare-combined: clean combine copy-defs

combine:
	@mkdir -p $(PREPAREDIR)
	@rm -f $(PREPAREDIR)/all.cpp
	@for src in $(SRCS_FILES); do \
		printf "\n\n// $$src\n" >> $(PREPAREDIR)/all.cpp; \
		cat $(SRCS_DIR)/$$src >> $(PREPAREDIR)/all.cpp; \
	done

copy-defs:
	@mkdir -p $(PREPAREDIR)
	@for hdr in $(HEADERS); do \
		cp $(SRCS_DIR)/$$hdr $(PREPAREDIR)/; \
	done

build-combined: prepare-combined
	@cp $(SRCS_DIR)/defs-test.h $(PREPAREDIR)/defs-test.h 2>/dev/null || true
	$(CXX) $(CXXFLAGS) $(LDFLAGS) -o $(TARGET) $(PREPAREDIR)/all.cpp

test: build
	Hyprland --config ./test-hyprland.conf

test-combined: build-combined
	Hyprland --config ./test-hyprland.conf

clean:
	@rm -f $(TARGET)
	@rm -rf $(PREPAREDIR)