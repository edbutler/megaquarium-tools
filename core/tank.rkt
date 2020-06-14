#lang racket

(provide
  (except-out
    (all-defined-out)
    tnktyp
    tank))

(module+ test
  (provide make-test-tnktyp)
  (require rackunit "../test.rkt"))

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

(struct tnktyp game-object-template
  (min-dimensions
   max-dimensions
   volume-per-tile
   ; boolean?
   rounded?)
  #:transparent)

(define tnktyp-id game-object-template-id)

(define (make-tnktyp
          #:id id
          #:min min-dim
          #:max max-dim
          #:density density
          #:rounded? rounded?)
  (local-require (only-in racket raise-argument-error symbol? positive-integer?))

  (define (err contract pos)
    (raise-argument-error 'make-tnktyp contract pos id min-dim max-dim density rounded?))
  (define (int-pair? v) (and (pair? v) (positive-integer? (car v)) (positive-integer? (cdr v))))

  (unless (symbol? id) (err "symbol?" 0))
  (unless (int-pair? min-dim) (err "(pairof positive-integer? positive-integer?)" 1))
  (unless (int-pair? max-dim) (err "(pairof positive-integer? positive-integer?)" 2))
  (unless (positive? density) (err "positive?" 3))
  (unless (boolean? rounded?) (err "boolean?" 4))

  (tnktyp id min-dim max-dim density rounded?))

(module+ test
  (let ()
    (define id 'someid)
    (define min-dim (cons 5 6))
    (define max-dim (cons 7 8))
    (define density 2)
    (define rounded? #t)

    (test-case "can create tnktyp"
      (define val (tnktyp id min-dim max-dim density rounded?))
      (check-eq? (tnktyp-id val) id)
      (check-eq? (tnktyp-min-dimensions val) min-dim)
      (check-eq? (tnktyp-max-dimensions val) max-dim)
      (check-eq? (tnktyp-volume-per-tile val) density)
      (check-eq? (tnktyp-rounded? val) rounded?))

    (test-case "can use make-tnktyp"
      (define val
        (make-tnktyp
          #:id id
          #:min min-dim
          #:max max-dim
          #:density density
          #:rounded? rounded?))
      (define expected (tnktyp id min-dim max-dim density rounded?))
      (check-equal? val expected)))

  (define (make-test-tnktyp
            #:id [id #f]
            #:min [min-dim (cons 2 2)]
            #:max [max-dim (cons 4 4)]
            #:density [density 3]
            #:rounded? [rounded? #f])
    (make-tnktyp
      #:id (or id (fresh-symbol))
      #:min min-dim
      #:max max-dim
      #:density density
      #:rounded? rounded?)))

; A fully specified tank.

(struct tank
   ; integer?
  (id
   ; string?: The name used in game
   name
   ; tnktyp?
   type
   ; positive-integer?
   size
   ; environment?
   environment
   ; nonnegative-integer?
   lighting)
  #:transparent)

(define (calculate-tank-size kind x-dim y-dim)
  (local-require (only-in racket exact-ceiling))
  (exact-ceiling (* x-dim y-dim (tnktyp-volume-per-tile kind))))

(define (make-tank
          #:id id
          #:name name
          #:kind kind
          #:dimensions [dim #f]
          #:size [size #f]
          #:environment env
          #:lighting light)
  (local-require
    (only-in racket raise-argument-error symbol? positive-integer? nonnegative-integer? string? error))

  (define (err contract pos)
    (raise-argument-error 'make-tank contract pos id name kind dim size env light))
  (define (int-pair? v) (and (pair? v) (positive-integer? (car v)) (positive-integer? (cdr v))))

  (unless (nonnegative-integer? id) (err "nonnegative-integer?" 0))
  (unless (string? name) (err "string?" 1))
  (unless (tnktyp? kind) (err "tnktyp?" 2))
  (unless (or dim size) (error "need to define either size or dimensions"))
  (unless (or (not dim) (int-pair? dim)) (err "(pairof positive-integer? positive-integer?" 3))
  (unless (or (not size) (positive-integer? size)) (err "positive-integer?" 4))
  (unless (environment? env) (err "environment?" 5))
  (unless (nonnegative-integer? light) (err "nonnegative-integer?" 6))

  (define sz (or size (calculate-tank-size kind (car dim) (cdr dim))))
  (tank id name kind sz env light))

(module+ test
  (let ()
    (define id 2534)
    (define name "A name")
    (define kind (make-test-tnktyp #:density 5))
    (define dim (cons 3 6))
    (define size 90)
    (define env (environment warm-water 50))
    (define lighting 15)

    (test-case "can create tank"
      (define val (tank id name kind size env lighting))
      (check-eq? (tank-id val) id)
      (check-eq? (tank-name val) name)
      (check-eq? (tank-type val) kind)
      (check-eq? (tank-size val) size)
      (check-eq? (tank-environment val) env)
      (check-eq? (tank-lighting val) lighting))

    (test-case "can use make-tank with dimensions"
      (define val
        (make-tank
          #:id id
          #:name name
          #:kind kind
          #:dimensions dim
          #:environment env
          #:lighting lighting))
      (define expected (tank id name kind size env lighting))
      (check-equal? val expected))

    (test-case "can use make-tank with size"
      (define val
        (make-tank
          #:id id
          #:name name
          #:kind kind
          #:size size
          #:environment env
          #:lighting lighting))
      (define expected (tank id name kind size env lighting))
      (check-equal? val expected))))
