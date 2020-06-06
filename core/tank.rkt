#lang rosette/safe

(provide (all-defined-out))

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

(struct tank-info
  (min-dimensions
   max-dimensions
   volume-per-tile
   ; boolean?
   rounded?)
  #:transparent)

; A fully specified tank.

(struct tank
   ; symbol?: The identifier for this tank
  (id
   ; string?: The name used in game
   name
   ; tank-info? (TODO clean this up)
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

; TODO unused
;(define (make-tank-of-type type temp x-dim y-dim)
;  (local-require (only-in racket exact-ceiling))
;  (define sz (exact-ceiling (* x-dim y-dim (tank-info-volume-per-tile type))))
;  (tank type sz temp))
