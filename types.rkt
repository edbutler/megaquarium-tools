#lang rosette/safe

; Definitions for all of aquarium-related data types

(provide (all-defined-out))

(require
  "tank.rkt"
  (only-in racket local-require))

(module+ test
  (require
    (only-in racket exn:fail:contract?)
    rackunit))

(struct species
   ; symbol?
  (id
   ; symbol? ('fish, 'coral', ...)
   class
   ; symbol? ('stony_coral, 'grouper, ...)
   type
   ; size?
   size
   ; environment?: the min environment required (desired temp, min quality)
   environment
   ; diet?: what this species eats
   diet
   ; (listof property?): properties of this fish that impact others
   properties
   ; (listof restriction?): restrictions for this fishes placement.
   ; all must hold for fish to be happy.
   restrictions
   ; unlockable?: when this fish can be researched
   unlockable)
  #:transparent)

; duration=#f for final stage
(struct size
   ; (listof species-stage?)
  (stages
   ; boolean?
   armored?)
  #:transparent)
(struct species-stage (size duration) #:transparent)

(struct diet () #:transparent)
(struct food diet (type period) #:transparent)
(struct scavenger diet () #:transparent)
(struct does-not-eat diet () #:transparent)

(struct property () #:transparent)
(struct bully property () #:transparent)

(struct restriction () #:transparent)
(struct shoaler restriction (count) #:transparent)
(struct active-swimmer restriction (multiplier) #:transparent)
; size=#f means means any size
(struct predator restriction (type size) #:transparent)
(struct dislikes-conspecifics restriction () #:transparent)
(struct dislikes-congeners restriction () #:transparent)
(struct only-congeners restriction () #:transparent)
(struct rounded-tank restriction () #:transparent)
(struct dislikes-food-competitors restriction () #:transparent)
(struct dislikes-light restriction () #:transparent)
(struct requires-light restriction (amount) #:transparent)
(struct wimp restriction () #:transparent)

(struct unlockable (rank) #:transparent)

(struct animal
  (id
   species)
  #:transparent)

; access helpers

(define (species-final-size s)
  (species-stage-size (last (size-stages (species-size s)))))

(define animal-species-id (compose1 species-id animal-species))
(define animal-class (compose1 species-class animal-species))
(define animal-type (compose1 species-type animal-species))
(define animal-environment (compose1 species-environment animal-species))
(define animal-properties (compose1 species-properties animal-species))
(define animal-restrictions (compose1 species-restrictions animal-species))
(define (animal-final-size a)
  (species-stage-size (last (size-stages (species-size (animal-species a))))))
(define (animal-armored? a)
  (size-armored? (species-size (animal-species a))))
(define animal-diet (compose1 species-diet animal-species))

(define (animal-required-food-amount a)
  (define f (animal-diet a))
  (cond
   [(food? f)
    ; this is wrong but correct-ish for small animals, have to figure out pattern for real
    (animal-final-size a)]
   [else 0]))

; build helpers

(define (mass-partition preds lst)
  (local-require (only-in racket match define-values))
  (match preds
   ['() (list lst)]
   [(cons head tail)
    (define-values (a b) (partition head lst))
    (cons a (mass-partition tail b))]))

; helper function to make it easier to define species
(define (make-fish id type . args)
  (local-require (only-in racket match-define raise-argument-error symbol? symbol->string))

  (define (err contract pos) (raise-argument-error 'make-fish contract pos id type args))

  (unless (symbol? id) (err "symbol?" 0))
  (unless (symbol? type) (err "symbol?" 1))

  (match-define (list stage-lst env-lst diet-lst properties others)
    (mass-partition (list size? environment? diet? property?) args))

  (unless (= 1 (length stage-lst)) (err "one (listof species-stage?)" 2))
  (unless (= 1 (length env-lst)) (err "one environment?" 2))
  (unless (= 1 (length diet-lst)) (err "one diet?" 2))
  (unless (andmap restriction? others) (err "others restriction?" 2))

  (define stages (first stage-lst))
  (define env (first env-lst))
  (define diet (first diet-lst))
  (define restrictions others)

  (species id
           'fish
           type
           stages
           env
           diet
           properties
           restrictions
           (unlockable 1)))

(module+ test
  (test-case "make-fish"
    (define fsh-1
      (make-fish 'bob
                    'fish
                    (environment warm-water 50)
                    (make-size 20)
                    (shoaler 5)
                    (food 'green 1)))
    (define fsh-2
      (make-fish 'bob
                    'fish
                    (shoaler 5)
                    (food 'green 1)
                    (make-size 20)
                    (environment warm-water 50)))
    (define fsh-3
      (make-fish 'bob
                    'fish
                    (food 'green 1)
                    (make-size 21)
                    (shoaler 5)
                    (environment warm-water 50)))
    (check-pred species? fsh-1)
    (check-pred species? fsh-2)
    (check-pred species? fsh-3)
    (check-equal? fsh-1 fsh-2)
    (check-not-equal? fsh-1 fsh-3))

(test-case "make-fish-errors"
  (define warm-env (environment warm-water 50))
  (check-exn exn:fail:contract? (thunk (make-fish "name" 'type warm-env (make-size 2) (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name "type" warm-env (make-size 2) (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type (make-size 2) (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type warm-env warm-env (make-size 2) (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type warm-env (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type warm-env (make-size 2) (make-size 2) (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type warm-env (make-size 2))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type warm-env (make-size 2) (food 'a 1) (food 'a 1))))
  (check-exn exn:fail:contract? (thunk (make-fish 'name 'type warm-env (make-size 2) (food 'a 1) 4)))))

(define (make-size n #:armored? [armored? #f]) (size (list (species-stage n #f)) armored?))
(define (make-active-swimmer) (active-swimmer 6))

; drop the excess stuff from the names, specifically the leading number
(define (tweak-species-name name)
  (local-require (only-in racket regexp-replace* symbol->string symbol?))
  (when (symbol? name) (set! name (symbol->string name)))
  (regexp-replace* #rx"[0-9]+_(.*)" name "\\1"))

(struct condition () #:transparent)
(struct species-condition condition (id quantity) #:transparent)
(struct restriction-condition condition (name quantity) #:transparent)
(struct type-condition condition (type quantity) #:transparent)

(struct objective (condition) #:transparent)

(struct market
  ; (listof symbol?)
  (available
  ; (listof symbol?)
   unlockable
  ; (listof (pairof symbol? positive-integer?))
   acquirable)
  #:transparent)

(struct aquarium
  ; (pairof tank? (listof animal?))
 (tanks
  ; market?
  market
  ; (listof objective?)
  objectives)
 #:transparent)

(struct species-spec
   ; species?
  (species
   ; integer?
   count)
  #:transparent)

(struct tank-spec
  (name
   ; (listof species-spec?)
   contents)
  #:transparent)
