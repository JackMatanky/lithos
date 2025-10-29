package main

import (
	"fmt"
	"mime"
)

func main() {
	fmt.Println("md:", mime.TypeByExtension(".md"))
	fmt.Println("txt:", mime.TypeByExtension(".txt"))
	fmt.Println("xyz:", mime.TypeByExtension(".xyz"))
	fmt.Println("json:", mime.TypeByExtension(".json"))
}
