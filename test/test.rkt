#lang errortrace racket

(require
  "../types.rkt"
  "../constraint.rkt"
  rackunit)

(define-simple-check (check-tank-ok? tnk animals)
  (define dom (make-concrete-domain (list (cons tnk animals))))
  (all-constraints-satisfied? dom))

(define-simple-check (check-tank-not-ok? tnk animals)
  (define dom (make-concrete-domain (list (cons tnk animals))))
  (not (all-constraints-satisfied? dom)))

(define warm-env (environment warm-water 50))
(define cold-env (environment cold-water 50))

(define basic-tank-info (tank-info (cons 2 2) (cons 6 6) 2 #f))
(define rounded-tank-info (tank-info (cons 2 2) (cons 6 6) 2 #t))
(define tank-warm-100 (make-tank 1 #:type basic-tank-info #:size 100 #:environment warm-env #:lighting 0))
(define tank-cold-100 (make-tank 2 #:type basic-tank-info #:size 100 #:environment cold-env #:lighting 0))
(define tank-warm-20  (make-tank 3 #:type basic-tank-info #:size  20 #:environment warm-env #:lighting 0))
(define tank-cold-20  (make-tank 4 #:type basic-tank-info #:size  20 #:environment cold-env #:lighting 0))

(define (make-fishes spec cnt)
  (define (name i) (format "~a-~a" (species-id spec) i))
  (build-list cnt (λ (i) (animal (name i) spec))))

(test-case "tank-temperature"
  (define warm (make-fish 'warm 'fish warm-env (make-size 15) (food 'x 1)))
  (define cold (make-fish 'cold 'fish cold-env (make-size 15) (food 'x 1)))
  (define warm-fishes (make-fishes warm 3))
  (define cold-fishes (make-fishes cold 3))
  (check-tank-ok? tank-warm-20 (take warm-fishes 1))
  (check-tank-not-ok? tank-cold-20 (take warm-fishes 1))
  (check-tank-ok? tank-cold-20 (take cold-fishes 1))
  (check-tank-not-ok? tank-warm-20 (take cold-fishes 1))
  (check-tank-ok? tank-warm-100 warm-fishes)
  (check-tank-ok? tank-cold-100 cold-fishes)
  (check-tank-not-ok? tank-warm-100 (append warm-fishes cold-fishes))
  (check-tank-not-ok? tank-cold-100 (append warm-fishes cold-fishes))
  (check-tank-not-ok? tank-warm-100 (append cold-fishes warm-fishes))
  (check-tank-not-ok? tank-cold-100 (append cold-fishes warm-fishes)))

(test-case "shoaling"
  (define spec (make-fish 'warm 'fish warm-env (make-size 2) (food 'x 1) (shoaler 5)))
  (define fishes (make-fishes spec 7))
  (check-tank-not-ok? tank-warm-20 (take fishes 1))
  (check-tank-not-ok? tank-warm-20 (take fishes 2))
  (check-tank-not-ok? tank-warm-20 (take fishes 3))
  (check-tank-not-ok? tank-warm-20 (take fishes 4))
  (check-tank-ok? tank-warm-20 (take fishes 5))
  (check-tank-ok? tank-warm-20 (take fishes 6))
  (check-tank-ok? tank-warm-20 (take fishes 7)))

(test-case "predation"
  (define pred (animal 0 (make-fish 'warm 'fish warm-env (make-size 20) (food 'x 1) (predator 'a 5) (predator 'b 5) (predator 'c #f))))
  (define pred-tiny (animal 0 (make-fish 'warm 'fish warm-env (make-size 20) (food 'x 1) (predator 'a 4) (predator 'b 4) (predator 'c #f))))
  (define prey-a-5 (animal 1 (make-fish 'warm 'a warm-env (make-size 5) (food 'x 1))))
  (define prey-a-6 (animal 2 (make-fish 'warm 'a warm-env (make-size 6) (food 'x 1))))
  (define prey-b-5 (animal 3 (make-fish 'warm 'b warm-env (make-size 5) (food 'x 1))))
  (define prey-b-6 (animal 4 (make-fish 'warm 'b warm-env (make-size 6) (food 'x 1))))
  (define prey-c-5 (animal 5 (make-fish 'warm 'c warm-env (make-size 5) (food 'x 1))))
  (define prey-c-6 (animal 6 (make-fish 'warm 'c warm-env (make-size 6) (food 'x 1))))

  (check-tank-ok? tank-warm-100 (list pred pred-tiny))
  (check-tank-ok? tank-warm-100 (list prey-a-5 prey-a-6 prey-b-5 prey-b-6 prey-c-5 prey-c-6))
  ; prey should be eaten if at most 5, not 6
  (check-tank-not-ok? tank-warm-100 (list pred prey-a-5))
  (check-tank-ok? tank-warm-100 (list pred prey-a-6))
  (check-tank-not-ok? tank-warm-100 (list pred prey-b-5))
  (check-tank-ok? tank-warm-100 (list pred prey-b-6))
  ; except c which should always be eaten
  (check-tank-not-ok? tank-warm-100 (list pred prey-c-5))
  (check-tank-not-ok? tank-warm-100 (list pred prey-c-6))
  ; tiny pred can't eat anything
  (check-tank-ok? tank-warm-100 (list pred-tiny prey-a-5))
  (check-tank-ok? tank-warm-100 (list pred-tiny prey-a-6))
  (check-tank-ok? tank-warm-100 (list pred-tiny prey-b-5))
  (check-tank-ok? tank-warm-100 (list pred-tiny prey-b-6))
  ; except c which should always be eaten
  (check-tank-not-ok? tank-warm-100 (list pred-tiny prey-c-5))
  (check-tank-not-ok? tank-warm-100 (list pred-tiny prey-c-6)))

(test-case "predation-armored"
  (define pred (animal 0 (make-fish 'warm 'fish warm-env (make-size 20) (food 'x 1) (predator 'a 10) (predator 'c #f))))
  (define pred-tiny (animal 0 (make-fish 'warm 'fish warm-env (make-size 20) (food 'x 1) (predator 'a 9) (predator 'c #f))))
  (define prey-a-5 (animal 1 (make-fish 'warm 'a warm-env (make-size 5 #:armored? #t) (food 'x 1))))
  (define prey-a-6 (animal 2 (make-fish 'warm 'a warm-env (make-size 6 #:armored? #t) (food 'x 1))))
  (define prey-c-5 (animal 1 (make-fish 'warm 'c warm-env (make-size 5 #:armored? #t) (food 'x 1))))
  (define prey-c-6 (animal 2 (make-fish 'warm 'c warm-env (make-size 6 #:armored? #t) (food 'x 1))))

  (check-tank-ok? tank-warm-100 (list pred pred-tiny))
  (check-tank-ok? tank-warm-100 (list prey-a-5 prey-a-6 prey-c-5 prey-c-6))
  ; amored prey should count for double
  (check-tank-not-ok? tank-warm-100 (list pred prey-a-5))
  (check-tank-ok? tank-warm-100 (list pred-tiny prey-a-5))
  (check-tank-ok? tank-warm-100 (list pred prey-a-6))
  (check-tank-ok? tank-warm-100 (list pred-tiny prey-a-6))
  ; c should still always be eaten
  (check-tank-not-ok? tank-warm-100 (list pred prey-c-5))
  (check-tank-not-ok? tank-warm-100 (list pred-tiny prey-c-5))
  (check-tank-not-ok? tank-warm-100 (list pred prey-c-6))
  (check-tank-not-ok? tank-warm-100 (list pred-tiny prey-c-6))

  (void))

(test-case "active-swimmer"
  (define swimmer (make-fish 'swimmy 'fish warm-env (make-size 6) (food 'x 1) (make-active-swimmer)))
  (define normie (make-fish 'noswimmy 'fish warm-env (make-size 6) (food 'x 1)))

  (define swimmers (make-fishes swimmer 6))
  (define normies (make-fishes normie 6))

  (define tank-warm-35 (make-tank 1 #:type basic-tank-info #:size 35 #:environment warm-env))
  (define tank-warm-36 (make-tank 2 #:type basic-tank-info #:size 36 #:environment warm-env))

  (check-tank-not-ok? tank-warm-20 (take swimmers 1))
  (check-tank-not-ok? tank-warm-35 (take swimmers 1))
  (check-tank-ok? tank-warm-36 (take swimmers 1))
  (check-tank-ok? tank-warm-36 swimmers)
  (check-tank-ok? tank-warm-100 (take swimmers 1))
  (check-tank-ok? tank-warm-100 swimmers)

  (check-tank-ok? tank-warm-20 (take normies 1))
  (check-tank-ok? tank-warm-35 (take normies 1))
  (check-tank-ok? tank-warm-36 (take normies 1))
  (check-tank-ok? tank-warm-36 normies)
  (check-tank-ok? tank-warm-100 (take normies 1))
  (check-tank-ok? tank-warm-100 normies))

(test-case "dislikes-conspecifics"
  (define loner (make-fish 'loner 'fish warm-env (make-size 2) (food 'x 1) (dislikes-conspecifics)))
  (define normie (make-fish 'normie 'fish warm-env (make-size 2) (food 'x 1)))
  (define loners (make-fishes loner 6))
  (define normies (make-fishes normie 6))

  (check-tank-ok? tank-warm-100 (take loners 1))
  (check-tank-not-ok? tank-warm-100 (take loners 2))
  (check-tank-not-ok? tank-warm-100 loners)
  (check-tank-ok? tank-warm-100 (append (take loners 1) normies))
  (check-tank-not-ok? tank-warm-100 (append (take loners 2) normies)))

(test-case "dislikes-congeners"
  (define loner (make-fish 'loner 'fish warm-env (make-size 2) (food 'x 1) (dislikes-congeners)))
  (define cousin (make-fish 'loner 'fish warm-env (make-size 2) (food 'x 1)))
  (define normie (make-fish 'normie 'notfish warm-env (make-size 2) (food 'x 1)))
  (define loners (make-fishes loner 6))
  (define cousins (make-fishes loner 6))
  (define normies (make-fishes normie 6))

  (check-tank-ok? tank-warm-100 (take loners 1))
  (check-tank-not-ok? tank-warm-100 (take loners 2))
  (check-tank-not-ok? tank-warm-100 loners)
  (check-tank-not-ok? tank-warm-100 (append (take loners 1) (take cousins 1)))
  (check-tank-not-ok? tank-warm-100 (append (take loners 2) cousins))
  (check-tank-ok? tank-warm-100 (append (take loners 1) normies))
  (check-tank-not-ok? tank-warm-100 (append (take loners 2) normies)))

(test-case "only-congeners"
  (define loner (make-fish 'loner 'fish warm-env (make-size 2) (food 'x 1) (only-congeners)))
  (define cousin (make-fish 'loner 'fish warm-env (make-size 2) (food 'x 1)))
  (define normie (make-fish 'normie 'notfish warm-env (make-size 2) (food 'x 1)))
  (define loners (make-fishes loner 6))
  (define cousins (make-fishes loner 6))
  (define normies (make-fishes normie 6))

  (check-tank-ok? tank-warm-100 (take loners 1))
  (check-tank-ok? tank-warm-100 (take loners 2))
  (check-tank-ok? tank-warm-100 loners)
  (check-tank-ok? tank-warm-100 (append (take loners 1) (take cousins 1)))
  (check-tank-ok? tank-warm-100 (append (take loners 2) cousins))
  (check-tank-not-ok? tank-warm-100 (append (take loners 1) (take normies 1)))
  (check-tank-not-ok? tank-warm-100 (append loners (take normies 1)))
  (check-tank-not-ok? tank-warm-100 (append (take loners 1) normies))
  (check-tank-not-ok? tank-warm-100 (append (take loners 2) normies)))

(test-case "needs-rounded-tank"
  (define roundie (make-fish 'a 'fish warm-env (make-size 2) (food 'x 1) (rounded-tank)))
  (define normie (make-fish 'a 'fish warm-env (make-size 2) (food 'x 1)))
  (define roundies (make-fishes roundie 7))
  (define normies (make-fishes normie 7))

  (define square-tank (make-tank 1 #:type   basic-tank-info #:size 100 #:environment warm-env))
  (define round-tank  (make-tank 2 #:type rounded-tank-info #:size 100 #:environment warm-env))

  (check-tank-ok? round-tank (take roundies 1))
  (check-tank-ok? round-tank roundies)
  (check-tank-not-ok? square-tank (take roundies 1))
  (check-tank-not-ok? square-tank roundies)

  (check-tank-ok? round-tank (take normies 1))
  (check-tank-ok? round-tank normies)
  (check-tank-ok? square-tank (take normies 1))
  (check-tank-ok? square-tank normies))

(test-case "dislikes-food-competitors"
  (define spec-x (make-fish 'spec-x 'fish warm-env (make-size 2) (food 'x 1) (dislikes-food-competitors)))
  (define comp-x (make-fish 'comp-x 'fish warm-env (make-size 2) (food 'x 1)))
  (define spec-y (make-fish 'spec-y 'fish warm-env (make-size 2) (food 'y 1) (dislikes-food-competitors)))
  (define comp-y (make-fish 'comp-y 'fish warm-env (make-size 2) (food 'y 1)))
  (define scav (make-fish 'scav 'fish warm-env (make-size 2) (scavenger)))

  (define spec-xs (make-fishes spec-x 6))
  (define comp-xs (make-fishes comp-x 6))
  (define spec-ys (make-fishes spec-y 6))
  (define comp-ys (make-fishes comp-y 6))
  (define scavs (make-fishes scav 6))

  (check-tank-ok? tank-warm-100 (take spec-xs 1))
  (check-tank-ok? tank-warm-100 spec-xs)
  (check-tank-not-ok? tank-warm-100 (append (take spec-xs 1) (take comp-xs 1)))
  (check-tank-not-ok? tank-warm-100 (append spec-xs comp-xs))
  (check-tank-ok? tank-warm-100 (append (take spec-xs 1) (take scavs 1)))
  (check-tank-ok? tank-warm-100 (append spec-xs scavs))

  (check-tank-ok? tank-warm-100 (take spec-ys 1))
  (check-tank-ok? tank-warm-100 spec-ys)
  (check-tank-not-ok? tank-warm-100 (append (take spec-ys 1) (take comp-ys 1)))
  (check-tank-not-ok? tank-warm-100 (append spec-ys comp-ys))
  (check-tank-ok? tank-warm-100 (append (take spec-ys 1) (take scavs 1)))
  (check-tank-ok? tank-warm-100 (append spec-ys scavs))

  (check-tank-ok? tank-warm-100 (append spec-xs comp-ys))
  (check-tank-ok? tank-warm-100 (append spec-ys comp-xs))
  (check-tank-ok? tank-warm-100 (append spec-xs spec-ys))
  (check-tank-not-ok? tank-warm-100 (append spec-xs spec-ys comp-ys))
  (check-tank-not-ok? tank-warm-100 (append spec-xs spec-ys comp-xs))
  (check-tank-ok? tank-warm-100 (append spec-xs spec-ys scavs)))

(test-case "dislikes-light"
  (define darklover (make-fish 'a 'fish warm-env (make-size 2) (food 'x 1) (dislikes-light)))
  (define normie (make-fish 'a 'fish warm-env (make-size 2) (food 'x 1)))
  (define darklovers (make-fishes darklover 7))
  (define normies (make-fishes normie 7))

  (define bright-tank (make-tank 1 #:type basic-tank-info #:size 100 #:environment warm-env #:lighting 1))
  (define dark-tank   (make-tank 2 #:type basic-tank-info #:size 100 #:environment warm-env #:lighting 0))

  (check-tank-ok? dark-tank (take darklovers 1))
  (check-tank-ok? dark-tank darklovers)
  (check-tank-not-ok? bright-tank (take darklovers 1))
  (check-tank-not-ok? bright-tank darklovers)

  (check-tank-ok? dark-tank (take normies 1))
  (check-tank-ok? dark-tank normies)
  (check-tank-ok? bright-tank (take normies 1))
  (check-tank-ok? bright-tank normies))

(test-case "requires-light"
  (define lightlover (make-fish 'a 'fish warm-env (make-size 2) (food 'x 1) (requires-light 10)))
  (define normie (make-fish 'a 'fish warm-env (make-size 2) (food 'x 1)))
  (define lightlovers (make-fishes lightlover 7))
  (define normies (make-fishes normie 7))

  (define bright-tank (make-tank 1 #:type basic-tank-info #:size 100 #:environment warm-env #:lighting 10))
  (define dim-tank    (make-tank 1 #:type basic-tank-info #:size 100 #:environment warm-env #:lighting 9))
  (define dark-tank   (make-tank 2 #:type basic-tank-info #:size 100 #:environment warm-env #:lighting 0))

  (check-tank-not-ok? dark-tank (take lightlovers 1))
  (check-tank-not-ok? dark-tank lightlovers)
  (check-tank-not-ok? dim-tank (take lightlovers 1))
  (check-tank-not-ok? dim-tank lightlovers)
  (check-tank-ok? bright-tank (take lightlovers 1))
  (check-tank-ok? bright-tank lightlovers)

  (check-tank-ok? dark-tank (take normies 1))
  (check-tank-ok? dark-tank normies)
  (check-tank-ok? dim-tank (take normies 1))
  (check-tank-ok? dim-tank normies)
  (check-tank-ok? bright-tank (take normies 1))
  (check-tank-ok? bright-tank normies))

(test-case "wimps-and-bullies"
  (define wimp-spec (make-fish 'wimp 'fish warm-env (make-size 2) (food 'x 1) (wimp)))
  (define bully-spec (make-fish 'bully 'fish warm-env (make-size 2) (food 'x 1) (bully)))
  (define wimps (make-fishes wimp-spec 6))
  (define bullies (make-fishes bully-spec 6))

  (check-tank-ok? tank-warm-100 wimps)
  (check-tank-ok? tank-warm-100 bullies)
  (check-tank-not-ok? tank-warm-100 (append wimps (take bullies 1)))
  (check-tank-not-ok? tank-warm-100 (append wimps bullies))
  (check-tank-not-ok? tank-warm-100 (append (take wimps 1) (take bullies 1)))
  (check-tank-not-ok? tank-warm-100 (append (take wimps 1) bullies)))

