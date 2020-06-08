#lang rosette/safe

(provide (all-defined-out))

(require
  (only-in racket local-require))

(module+ test
  (require
    rackunit))

(define warm-water 'warm-water)
(define cold-water 'cold-water)
(define (temperature? t) (or (equal? t warm-water) (equal? t cold-water)))

; TODO unused
(define salt-water 'salt-water)
(define fresh-water 'fresh-water)
(define (salinity? t) (or (equal? t salt-water) (equal? t fresh-water)))

(struct environment
  ; temperature?
  (temperature
   ; nonnegative-integer? (between 0 and 100)
   quality)
  #:transparent)

(module+ test
  (test-case "can create environment"
    (define temp cold-water)
    (define quality 85)
    (define val (environment temp quality))
    (check-eq? (environment-temperature val) temp)
    (check-eq? (environment-quality val) quality))

)

(struct game-object-template
  ; symbol?
  (id)
  #:transparent)

(struct tank-kind game-object-template
  (min-dimensions
   max-dimensions
   volume-per-tile
   ; boolean?
   rounded?)
  #:transparent)

(define tank-kind-id game-object-template-id)

(module+ test
  (test-case "can create tank-kind"
    (define id 'someid)
    (define min-dim (cons 5 6))
    (define max-dim (cons 7 8))
    (define density 2)
    (define rounded? #t)

    (define val (tank-kind id min-dim max-dim density rounded?))
    (check-eq? (tank-kind-id val) id)
    (check-eq? (tank-kind-min-dimensions val) min-dim)
    (check-eq? (tank-kind-max-dimensions val) max-dim)
    (check-eq? (tank-kind-volume-per-tile val) density)
    (check-eq? (tank-kind-rounded? val) rounded?)))

(define (make-tank-kind
          id
          #:min min-dim
          #:max max-dim
          #:density density
          #:rounded? rounded?)
  (local-require (only-in racket raise-argument-error symbol? positive-integer?))

  (define (err contract pos)
    (raise-argument-error 'make-tank-kind contract pos id min-dim max-dim density rounded?))
  (define (int-pair? v) (and (pair? v) (positive-integer? (car v)) (positive-integer? (cdr v))))

  (unless (symbol? id) (err "symbol?" 0))
  (unless (int-pair? min-dim) (err "(pairof positive-integer? positive-integer?)" 1))
  (unless (int-pair? max-dim) (err "(pairof positive-integer? positive-integer?)" 2))
  (unless (positive? density) (err "positive?" 3))
  (unless (boolean? rounded?) (err "boolean?" 4))

  (tank-kind id min-dim max-dim density rounded?))

(module+ test
  (test-case "can use make-tank-kind"
    (define id 'someid)
    (define min-dim (cons 5 6))
    (define max-dim (cons 7 8))
    (define density 2)
    (define rounded? #t)

    (define val
      (make-tank-kind
        id
        #:min min-dim
        #:max max-dim
        #:density density
        #:rounded? rounded?))

    (define expected (tank-kind id min-dim max-dim density rounded?))
    (check-equal? val expected)))

; A fully specified tank.

(struct tank
   ; symbol?: The identifier for this tank
  (id
   ; string?: The name used in game
   name
   ; tank-kind? (TODO clean this up)
   type
   ; positive-integer?
   size
   ; environment?
   environment
   ; nonnegative-integer?
   lighting)
  #:transparent)

(define (make-tank
          [id #f]
          #:name [name #f]
          #:type [type #f]
          #:size size
          #:environment env
          #:lighting [light 0])
  (tank (or id 0) (or name "unnamed-tank") type size env light))

(module+ test
  (test-case "can create tank"
    (define id 'someid)
    (define name "A name")
    (define type (void))
    (define size 45)
    (define env (environment warm-water 50))
    (define lighting 15)

    (define val (tank id name type size env lighting))
    (check-eq? (tank-id val) id)
    (check-eq? (tank-name val) name)
    (check-eq? (tank-type val) type)
    (check-eq? (tank-size val) size)
    (check-eq? (tank-environment val) env)
    (check-eq? (tank-lighting val) lighting))
  
  )

(define (calculate-tank-size kind x-dim y-dim)
  (local-require (only-in racket exact-ceiling))
  (exact-ceiling (* x-dim y-dim (tank-kind-volume-per-tile kind))))

; TODO unused
;(define (make-tank-of-type type temp x-dim y-dim)
;  (local-require (only-in racket exact-ceiling))
;  (define sz (exact-ceiling (* x-dim y-dim (tank-kind-volume-per-tile type))))
;  (tank type sz temp))
