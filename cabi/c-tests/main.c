#include <stdio.h>
#include <time.h>
#include "memento.h"

int main(int argc, char **argv) {
    time_t now = time(NULL);
    MementoResult res = memento_result_fetch("../tests/count_01.wsp", 100, now);

    if (memento_result_is_error(&res)) {
        fprintf(stderr, "Failure getting results!\n");
        memento_result_free(&res);
        return 1;
    }

    for (int i = 0; i < res.size; i++) {
        printf("%u: %f\n", res.results[i].timestamp, res.results[i].value);
    }

    memento_result_free(&res);
    return 0;
}
