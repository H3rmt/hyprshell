all:
	gcc -shared -fPIC --no-gnu-unique -o hyprshell.so -std=c++2b -O2 -I/usr/include/pixman-1 exit.cpp init.cpp key-press.cpp layer-change.cpp main.cpp

single:
	gcc -shared -fPIC --no-gnu-unique -o hyprshell.so -std=c++2b -O2 -I/usr/include/pixman-1 all.cpp

clean:
	rm ./hyprshell.so