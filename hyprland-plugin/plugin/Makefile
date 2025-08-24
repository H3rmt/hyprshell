.PHONY: all clean combine prepare-combined copy-defs build build-combined

SRCS_DIR = ./src
SRCS = key-press.cpp layer-change.cpp init.cpp exit.cpp main.cpp send.cpp
PREPAREDIR = out
OUTNAME = hyprshell.so
TARGET = $(OUTNAME)
HEADERS = globals.h handlers.h defs.h send.h

CXXFLAGS = -shared -fPIC --no-gnu-unique -std=c++2b -O2 -I/usr/include/pixman-1

prepare-combined: combine copy-defs

combine:
# Combine all source files into one for easier compilation
	mkdir -p $(PREPAREDIR)
	rm -f $(PREPAREDIR)/all.cpp
	for src in $(SRCS); do \
		printf "\n\n// $$src \n" >> $(PREPAREDIR)/all.cpp; \
		cat $(SRCS_DIR)/$$src >> $(PREPAREDIR)/all.cpp; \
	done

copy-defs:
# Copy header files to the out directory
	for src in $(HEADERS); do \
    	cp $(SRCS_DIR)/$$src $(PREPAREDIR)/; \
    done

build:
# Compile the combined source file into a shared library
	g++ $(CXXFLAGS) -o $(TARGET) $(SRCS_DIR)/*.cpp

test: build
# Run Hyprland in dev mode with the compiled plugin
	Hyprland --config ./test-hyprland.conf

build-combined: prepare-combined
# Build all.cpp
	cp $(SRCS_DIR)/defs-test.h $(PREPAREDIR)/defs-test.h
	g++ $(CXXFLAGS) -o $(TARGET) $(PREPAREDIR)/all.cpp

test-combined: build-combined
# Run Hyprland in dev mode with the compiled plugin
	Hyprland --config ./test-hyprland.conf
	

clean:
	rm -f $(TARGET)
	rm -rf $(PREPAREDIR)
