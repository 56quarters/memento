#include <stdio.h>
#include <time.h>
#include "memento.h"

static void print_results(MementoPointsResult *res) {
    for (int i = 0; i < res->size; i++) {
        printf("%u: %f\n", res->points[i].timestamp, res->points[i].value);
    }

}

static void print_header(MementoHeaderResult *res) {
    printf("Aggregation: %u\n", res->header->metadata.aggregation);
    printf("Max retention: %u\n", res->header->metadata.max_retention);
    printf("X Files Factor: %f\n", res->header->metadata.x_files_factor);

    for (int i = 0; i < res->header->size; i++) {
        printf("Archive %i\n", i);
        printf("  Offset: %u\n", res->header->archives[i].offset);
        printf("  Seconds per point: %u\n", res->header->archives[i].seconds_per_point);
        printf("  Num points: %u\n", res->header->archives[i].num_points);
    }
}

int main(int argc, char **argv) {
    time_t now = time(NULL);
    MementoPointsResult *res1 = memento_points_fetch("../tests/count_01.wsp", 100, now);

    if (memento_points_is_error(res1)) {
        fprintf(stderr, "Failure getting results!\n");
        memento_points_free(res1);
        return 1;
    }

    print_results(res1);
    memento_points_free(res1);
    printf("\n");

    MementoHeaderResult *res2 = memento_header_fetch("../tests/count_01.wsp");

    if (memento_header_is_error(res2)) {
        fprintf(stderr, "Failure getting header!\n");
        memento_header_free(res2);
        return 1;
    }

    print_header(res2);
    memento_header_free(res2);

    return 0;
}
