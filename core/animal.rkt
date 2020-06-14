#lang racket

(require
  "tank.rkt")

(module+ test
  (require rackunit))

; duration=#f for final stage
(struct size
   ; (listof species-stage?)
  (stages
   ; boolean?
   armored?)
  #:transparent)
(struct species-stage (size duration) #:transparent)

(provide
  (contract-out
   [struct size ((stages (listof species-stage?))
                 (armored? boolean?))]
   [struct species-stage ((size exact-nonnegative-integer?)
                          (duration (or/c #f exact-positive-integer?)))]))

(struct diet () #:transparent)
(struct food diet (type period) #:transparent)
(struct scavenger diet () #:transparent)
(struct does-not-eat diet () #:transparent)

(provide (struct-out diet)
         (struct-out food)
         (struct-out scavenger)
         (struct-out does-not-eat))

(struct property () #:transparent)
(struct bully property () #:transparent)

(provide (struct-out property)
         (struct-out bully))

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

(provide (struct-out restriction)
         (struct-out shoaler)
         (struct-out active-swimmer)
         (struct-out predator)
         (struct-out dislikes-conspecifics)
         (struct-out dislikes-congeners)
         (struct-out only-congeners)
         (struct-out rounded-tank)
         (struct-out dislikes-food-competitors)
         (struct-out dislikes-light)
         (struct-out requires-light)
         (struct-out wimp))

(struct species
   ; symbol?
   ; symbol? ('fish, 'coral', ...)
  (id
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

(provide (struct-out species) species-id)

; TODO why is this here?
(struct unlockable (rank) #:transparent)

(provide (struct-out unlockable))

(struct animal
  (id
   species)
  #:transparent)

(provide (struct-out animal))

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

(provide
  species-final-size
  animal-species-id
  animal-class
  animal-type
  animal-environment
  animal-properties
  animal-restrictions
  animal-final-size
  animal-armored?
  animal-diet)


(define (animal-required-food-amount a)
  (define f (animal-diet a))
  (cond
   [(food? f)
    ; this is wrong but correct-ish for small animals, have to figure out pattern for real
    (animal-final-size a)]
   [else 0]))

(provide animal-required-food-amount)

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

(provide make-fish)

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

(provide make-size make-active-swimmer)
