// Simple test to verify init function concept
#include <stdio.h>

// This would be our dummy init function
void _init(void) {
    // Do nothing - just satisfy UPX requirement
}

int main() {
    printf("Init function test\n");
    return 0;
}