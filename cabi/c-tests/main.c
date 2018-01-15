#include <stdio.h>
#include <time.h>
#include "memento.h"

static void print_results(MementoPointsResult *res) {
    for (int i = 0; i < res->size; i++) {
        printf("%u: %f\n", res->points[i].timestamp, res->points[i].value);
    }

}

int main(int argc, char **argv) {
    time_t now = time(NULL);
    MementoPointsResult *res = memento_points_fetch("../tests/count_01.wsp", 100, now);

    if (memento_points_is_error(res)) {
        fprintf(stderr, "Failure getting results!\n");
        memento_points_free(res);
        return 1;
    }

    print_results(res);
    memento_points_free(res);

    return 0;
}
